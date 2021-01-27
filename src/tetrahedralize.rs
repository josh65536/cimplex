use crate::iter;
use crate::{
    tet::{HasTets, TetId},
    vertex::{HasPosition3D, Position, VertexId},
};
use float_ord::FloatOrd;
use fnv::FnvHashSet;
use nalgebra::{dimension::U3, Point1, Vector3};
use simplicity as sim;
use typenum::B1;

fn index_fn<M>(mesh: &M, i: VertexId) -> Vector3<f64>
where
    M: HasPosition3D,
    M::V: Position<Dim = U3>,
{
    mesh.position(i).coords
}

/// Modified in-sphere test to deal with the ghost vertex.
/// `m` is the point to test the in-sphere of; it cannot be the ghost.
fn in_sphere_with_ghosts<M>(mesh: &M, tet: TetId, m: VertexId, ghost: VertexId) -> bool
where
    M: HasTets<MwbT = B1> + HasPosition3D,
    M::V: Position<Dim = U3>,
{
    if tet.contains_vertex(ghost) {
        let tri = tet.opp_tri(ghost);
        sim::orient_3d(mesh, index_fn, tri.0[0], tri.0[1], tri.0[2], m)
    } else {
        sim::in_sphere(mesh, index_fn, tet.0[0], tet.0[1], tet.0[2], tet.0[3], m)
    }
}

fn find_tet_to_delete<M>(mesh: &M, new_vertex: VertexId, ghost: VertexId) -> TetId
where
    M: HasTets<MwbT = B1> + HasPosition3D,
    M::V: Position<Dim = U3>,
{
    // Look for closest vertex to the new vertex to add
    let mut vertex = (mesh.tets().next().unwrap().0).0[0];
    while let Some(closer) = mesh
        .vertex_targets(vertex)
        .filter(|target| {
            mesh.distance_squared(*target, new_vertex) < mesh.distance_squared(vertex, new_vertex)
        })
        .min_by_key(|target| FloatOrd(mesh.distance_squared(*target, new_vertex)))
    {
        vertex = closer;
    }

    // The new vertex is in the circumsphere of some tet on that vertex.
    // If not, there's a floating-point error and we search further.

    iter::bfs(
        mesh.vertex_tets(vertex),
        |tet| mesh.adjacent_tets(*tet),
        |_| true,
    )
    .find(|tet| in_sphere_with_ghosts(mesh, *tet, new_vertex, ghost))
    .unwrap()
}

fn tets_to_delete<'a, M>(
    mesh: &'a M,
    new_vertex: VertexId,
    ghost: VertexId,
) -> impl Iterator<Item = TetId> + 'a
where
    M: HasTets<MwbT = B1> + HasPosition3D,
    M::V: Position<Dim = U3>,
{
    iter::bfs(
        std::iter::once(find_tet_to_delete(mesh, new_vertex, ghost)),
        move |tet| mesh.adjacent_tets(*tet),
        move |tet| in_sphere_with_ghosts(mesh, *tet, new_vertex, ghost),
    )
}

/// Implementation of the Bowyer-Watson algorithm,
/// with ghost tetrahedrons 👻 (https://people.eecs.berkeley.edu/~jrs/meshpapers/delnotes.pdf, section 3.4)
/// to avoid the concave tetrahedralization problem that happens with a super tet.
pub(crate) fn delaunay_tets<M>(mut mesh: M) -> M
where
    M: HasTets<MwbT = B1> + HasPosition3D,
    M::V: Position<Dim = U3>,
{
    // It takes 4 vertices to make a tet
    if mesh.num_vertices() < 4 {
        return mesh;
    }

    let mut v_ids = mesh.vertex_ids().copied().collect::<Vec<_>>();

    // Ghost vertex
    let ghost = mesh.add_with_position(Point1::new(f64::INFINITY).xxx());

    // First tet
    let v0 = v_ids.pop().unwrap();
    let v1 = v_ids.pop().unwrap();
    let mut v2 = v_ids.pop().unwrap();
    let mut v3 = v_ids.pop().unwrap();
    if !sim::orient_3d(&mesh, index_fn, v0, v1, v2, v3) {
        std::mem::swap(&mut v2, &mut v3);
    }
    let first = TetId::from_valid([v0, v1, v2, v3]);
    mesh.add_tet([v0, v1, v2, v3], mesh.default_tet());

    // Ghost tets
    for tri in &first.tris() {
        mesh.add_tet([tri.0[0], tri.0[2], tri.0[1], ghost], mesh.default_tet());
    }

    while let Some(vertex) = v_ids.pop() {
        let to_delete = tets_to_delete(&mesh, vertex, ghost).collect::<Vec<_>>();

        // Get boundary
        let tris = to_delete.iter().flat_map(|tet| tet.tris().to_vec()).collect::<FnvHashSet<_>>();
        let boundary = tris
            .iter()
            .copied()
            .filter(|tri| !tris.contains(&tri.twin()))
            .collect::<Vec<_>>();

        // Retetrahedralize region
        mesh.remove_tets(to_delete);
        mesh.extend_tets(
            boundary.into_iter()
                .map(|tri| {
                    (
                        TetId::from_valid([tri.0[0], tri.0[1], tri.0[2], vertex]),
                        mesh.default_tet(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    mesh.remove_vertex(ghost);
    mesh
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::fmt::Debug;
    use std::hash::Hash;

    use fnv::FnvHashSet;
    use nalgebra::Point3;

    use super::*;
    use crate::{ComboMesh0, mesh3::MwbComboMesh3};
    use crate::vertex::HasVertices;

    #[track_caller]
    fn assert_tets_m<
        V,
        E,
        F,
        T: Clone + Debug + Eq + Hash,
        TI: TryInto<TetId>,
        I: IntoIterator<Item = (TI, T)>,
    >(
        mesh: &MwbComboMesh3<V, E, F, T>,
        tets: I,
    ) {
        let result = mesh
            .tets()
            .map(|(id, f)| (*id, f.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = tets
            .into_iter()
            .map(|(vertices, f)| (vertices.try_into().ok().unwrap(), f))
            .collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
        assert_eq!(mesh.num_tets(), expect.len());
    }

    #[test]
    fn test_in_sphere_ghost() {
        let mut mesh = MwbComboMesh3::<Point3<f64>, (), (), ()>::with_defaults(
            || Point3::origin(),
            || (),
            || (),
            || (),
        );
        let ids = mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.5, 0.3, 0.6),
        ]);
        let tet = TetId::from_valid([ids[0], ids[1], ids[3], ids[2]]);
        mesh.add_tet(tet, ());

        assert!(!in_sphere_with_ghosts(&mesh, tet, ids[4], ids[0]));
        assert!(in_sphere_with_ghosts(&mesh, tet, ids[4], ids[1]));
        assert!(in_sphere_with_ghosts(&mesh, tet, ids[4], ids[2]));
        assert!(in_sphere_with_ghosts(&mesh, tet, ids[4], ids[3]));
    }

    #[test]
    fn test_in_sphere_no_ghost() {
        let mut mesh = MwbComboMesh3::<Point3<f64>, (), (), ()>::with_defaults(
            || Point3::origin(),
            || (),
            || (),
            || (),
        );
        let ids = mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.5, 0.3, 0.6),
        ]);
        let tet = TetId::from_valid([ids[0], ids[1], ids[3], ids[2]]);
        mesh.add_tet(tet, ());

        assert!(in_sphere_with_ghosts(&mesh, tet, ids[4], VertexId(5)));
    }

    #[test]
    fn test_tets_to_delete() {
        let mut mesh = MwbComboMesh3::<Point3<f64>, (), (), ()>::with_defaults(
            || Point3::origin(),
            || (),
            || (),
            || (),
        );
        let ids = mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            Point1::new(f64::INFINITY).xxx(),
            Point3::new(0.1, 0.1, 0.1),
            Point3::new(0.1, -0.1, 0.1),
            Point3::new(-1.0, -1.0, 0.1),
        ]);
        mesh.extend_tets(vec![
            ([ids[0], ids[1], ids[3], ids[2]], ()),
            ([ids[4], ids[1], ids[2], ids[3]], ()),
            ([ids[5], ids[0], ids[1], ids[3]], ()),
            ([ids[5], ids[3], ids[2], ids[0]], ()),
            ([ids[5], ids[1], ids[0], ids[2]], ()),
            ([ids[5], ids[4], ids[1], ids[2]], ()),
            ([ids[5], ids[2], ids[3], ids[4]], ()),
            ([ids[5], ids[1], ids[4], ids[3]], ()),
        ]);

        // In convex hull
        let result = tets_to_delete(&mesh, ids[6], ids[5]).collect::<FnvHashSet<_>>();
        assert_eq!(
            result,
            vec![
                TetId::from_valid([ids[0], ids[1], ids[3], ids[2]]),
                TetId::from_valid([ids[4], ids[1], ids[2], ids[3]]),
            ]
            .into_iter()
            .collect::<FnvHashSet<_>>()
        );

        // Remove both solid tetrahedrons and ghost tetrahedrons
        let result = tets_to_delete(&mesh, ids[7], ids[5]).collect::<FnvHashSet<_>>();
        assert_eq!(
            result,
            vec![
                TetId::from_valid([ids[0], ids[1], ids[3], ids[2]]),
                TetId::from_valid([ids[4], ids[1], ids[2], ids[3]]),
                TetId::from_valid([ids[5], ids[0], ids[1], ids[3]]),
            ]
            .into_iter()
            .collect::<FnvHashSet<_>>()
        );

        // Remove only ghost tetrahedrons
        let result = tets_to_delete(&mesh, ids[8], ids[5]).collect::<FnvHashSet<_>>();
        assert_eq!(
            result,
            vec![
                TetId::from_valid([ids[5], ids[0], ids[1], ids[3]]),
                TetId::from_valid([ids[5], ids[3], ids[2], ids[0]]),
            ]
            .into_iter()
            .collect::<FnvHashSet<_>>()
        );
    }

    #[test]
    fn test_delaunay_tets_empty() {
        let mut mesh = ComboMesh0::<Point3<f64>>::with_defaults(|| Point3::origin());
        mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = mesh.clone().delaunay_tets(|| (), || (), || ());
        assert_eq!(
            mesh.vertex_ids().collect::<FnvHashSet<_>>(),
            result.vertex_ids().collect::<FnvHashSet<_>>(),
        );
        assert_tets_m(&result, vec![] as Vec<(TetId, ())>);
    }

    #[test]
    fn test_delaunay_tets_single() {
        let mut mesh = ComboMesh0::<Point3<f64>>::with_defaults(|| Point3::origin());
        let ids = mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        ]);

        let result = mesh.clone().delaunay_tets(|| (), || (), || ());
        assert_eq!(
            mesh.vertex_ids().collect::<FnvHashSet<_>>(),
            result.vertex_ids().collect::<FnvHashSet<_>>(),
        );
        assert_tets_m(&result, vec![
            (TetId::from_valid([ids[0], ids[1], ids[3], ids[2]]), ()),
        ]);
    }

    #[test]
    fn test_delaunay_tets_multiple() {
        let mut mesh = ComboMesh0::<Point3<f64>>::with_defaults(|| Point3::origin());
        let ids = mesh.extend_vertices(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.5, 1.5, 1.0),
            Point3::new(0.5, 0.5, 0.5),
        ]);

        let result = mesh.clone().delaunay_tets(|| (), || (), || ());
        assert_eq!(
            mesh.vertex_ids().collect::<FnvHashSet<_>>(),
            result.vertex_ids().collect::<FnvHashSet<_>>(),
        );
        assert_tets_m(&result, vec![
            (TetId::from_valid([ids[0], ids[3], ids[2], ids[5]]), ()),
            (TetId::from_valid([ids[0], ids[1], ids[3], ids[5]]), ()),
            (TetId::from_valid([ids[1], ids[0], ids[2], ids[5]]), ()),
            (TetId::from_valid([ids[1], ids[4], ids[3], ids[5]]), ()),
            (TetId::from_valid([ids[3], ids[4], ids[2], ids[5]]), ()),
            (TetId::from_valid([ids[2], ids[4], ids[1], ids[5]]), ()),
        ]);
    }

    //#[test]
    //fn test_export() {
    //    let mut mesh = ComboMesh0::<Point3<f64>>::with_defaults(|| Point3::origin());
    //    mesh.extend_vertices((0..3).flat_map(|x| (0..3).flat_map(move |y| (0..3).map(move |z| 
    //        Point3::new(x as f64, y as f64, z as f64)))));

    //    let result = mesh.delaunay_tets(|| (), || (), || ());
    //    result.to_separate_tets().write_obj("assets/grid.obj").unwrap();
    //}
}

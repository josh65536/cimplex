use fnv::FnvHashMap;
use idmap::OrderedIdMap;
use nalgebra::dimension::{U2, U3};
use std::fmt::Debug;
use typenum::{B0, B1};

use crate::mesh2::internal::HigherEdge;
use crate::tet::{HasTets, TetId};
use crate::tri::{HasTris, TriId};
use crate::vertex::{HasVertices, VertexId};
use crate::PtN;
use crate::{
    edge::{EdgeId, HasEdges},
    vertex::IdType,
};
use crate::{mesh1::internal::HigherVertex, private::Lock};
use crate::{
    mesh1::MwbComboMesh1,
    mesh2::{ComboMesh2, MwbComboMesh2},
    ComboMesh0, ComboMesh1,
};

use internal::{HigherTri, MwbTet, Tet};

/// A combinatorial simplicial 3-complex, containing only vertices, (oriented) edges, (oriented) triangles, and (oriented) tetrahedrons.
/// Also known as an tet mesh.
/// Each vertex stores a value of type `V`.
/// Each edge stores its vertices and a value of type `E`.
/// Each triangle stores its vertices and a value of type `F`.
/// Each tetrahedron stores its vertices and a value of type `T`.
/// The edge manipulation methods can either be called with an array of 2 `VertexId`s
/// or an `EdgeId`.
/// The triangle manipulation methods can either be called with an array of 3 `VertexId`s
/// or an `TriId`.
/// The tetrahedron manipulation methods can either be called with an array of 4 `VertexId`s
/// or an `TetId`.
#[derive(Clone, Debug)]
pub struct ComboMesh3<V, E, F, T> {
    vertices: OrderedIdMap<VertexId, HigherVertex<V>>,
    edges: FnvHashMap<EdgeId, HigherEdge<E>>,
    tris: FnvHashMap<TriId, HigherTri<F>>,
    tets: FnvHashMap<TetId, Tet<T>>,
    next_vertex_id: IdType,
    default_v: fn() -> V,
    default_e: fn() -> E,
    default_f: fn() -> F,
    default_t: fn() -> T,
}
crate::impl_index_vertex!(ComboMesh3<V, E, F, T>);
crate::impl_index_edge!(ComboMesh3<V, E, F, T>);
crate::impl_index_tri!(ComboMesh3<V, E, F, T>);
crate::impl_index_tet!(ComboMesh3<V, E, F, T>);
crate::impl_with_eft!(ComboMesh3<V, E, F, T>: <V, E, F, T> ComboMesh1<V, E>, <V, E, F, T> ComboMesh2<V, E, F>, <V, E, F, T> ComboMesh3<V, E, F, T>);

impl<V, E, F, T> HasVertices for ComboMesh3<V, E, F, T> {
    crate::impl_has_vertices!(HigherVertex<V> zeroed zeroed zeroed, Higher = B1);

    fn remove_vertex_higher<L: Lock>(&mut self, vertex: VertexId) {
        self.remove_edges(
            self.vertex_edges_out(vertex)
                .chain(self.vertex_edges_in(vertex))
                .collect::<Vec<_>>(),
        );
    }

    fn clear_vertices_higher<L: Lock>(&mut self) {
        self.tets.clear();
        self.tris.clear();
        self.edges.clear();
    }
}

impl<V, E, F, T> HasEdges for ComboMesh3<V, E, F, T> {
    crate::impl_has_edges!(HigherEdge<E> zeroed zeroed, Mwb = B0, Higher = B1);

    type WithoutEdges = ComboMesh0<V>;
    type WithMwbE = MwbComboMesh1<V, E>;
    type WithoutMwbE = ComboMesh1<V, E>;

    fn remove_edge_higher<L: Lock>(&mut self, edge: EdgeId) {
        self.remove_tris_keep_edges(self.edge_tris(edge).collect::<Vec<_>>());
    }

    fn clear_edges_higher<L: Lock>(&mut self) {
        self.tets.clear();
        self.tris.clear();
    }
}

impl<V, E, F, T> HasTris for ComboMesh3<V, E, F, T> {
    crate::impl_has_tris!(HigherTri<F> zeroed, Mwb = B0, Higher = B1);

    type WithoutTris = ComboMesh1<V, E>;
    type WithMwbF = MwbComboMesh2<V, E, F>;
    type WithoutMwbF = ComboMesh2<V, E, F>;

    fn remove_tri_higher<L: Lock>(&mut self, tri: TriId) {
        self.remove_tets_keep_tris(self.tri_tets(tri).collect::<Vec<_>>());
    }

    fn clear_tris_higher<L: Lock>(&mut self) {
        self.tets.clear();
    }
}

impl<V, E, F, T> HasTets for ComboMesh3<V, E, F, T> {
    crate::impl_has_tets!(Tet<T>, Mwb = B0);

    type WithoutTets = ComboMesh2<V, E, F>;
    type WithMwbT = MwbComboMesh3<V, E, F, T>;
    type WithoutMwbT = ComboMesh3<V, E, F, T>;

    fn remove_tet_higher<L: Lock>(&mut self, _: TetId) {}

    fn clear_tets_higher<L: Lock>(&mut self) {}
}

impl<V: Default, E: Default, F: Default, T: Default> Default for ComboMesh3<V, E, F, T> {
    fn default() -> Self {
        ComboMesh3 {
            vertices: OrderedIdMap::default(),
            edges: FnvHashMap::default(),
            tris: FnvHashMap::default(),
            tets: FnvHashMap::default(),
            next_vertex_id: 0,
            default_v: Default::default,
            default_e: Default::default,
            default_f: Default::default,
            default_t: Default::default,
        }
    }
}

impl<V, E, F, T> ComboMesh3<V, E, F, T> {
    /// Creates an empty tet mesh.
    pub fn new() -> Self
    where
        V: Default,
        E: Default,
        F: Default,
        T: Default,
    {
        Self::default()
    }

    /// Creates an empty tet mesh with default values for elements.
    pub fn with_defaults(
        vertex: fn() -> V,
        edge: fn() -> E,
        tri: fn() -> F,
        tet: fn() -> T,
    ) -> Self {
        Self {
            vertices: OrderedIdMap::default(),
            edges: FnvHashMap::default(),
            tris: FnvHashMap::default(),
            tets: FnvHashMap::default(),
            next_vertex_id: 0,
            default_v: vertex,
            default_e: edge,
            default_f: tri,
            default_t: tet,
        }
    }
}

/// A position-containing tet mesh
pub type Mesh3<V, E, F, T, D> = ComboMesh3<(PtN<D>, V), E, F, T>;

/// A 2D-position-containing tet mesh
pub type Mesh32<V, E, F, T> = Mesh3<V, E, F, T, U2>;

/// A 3D-position-containing tet mesh
pub type Mesh33<V, E, F, T> = Mesh3<V, E, F, T, U3>;

/// A combinatorial simplicial 3-complex with the mwb property,
/// which forces every oriented triangle to be part of at most 1 tetrahedron.
/// Please don't call `add_edge` or `add_tri` on this.
#[derive(Clone, Debug)]
pub struct MwbComboMesh3<V, E, F, T> {
    vertices: OrderedIdMap<VertexId, HigherVertex<V>>,
    edges: FnvHashMap<EdgeId, HigherEdge<E>>,
    tris: FnvHashMap<TriId, HigherTri<F>>,
    tets: FnvHashMap<TetId, MwbTet<T>>,
    next_vertex_id: IdType,
    default_v: fn() -> V,
    default_e: fn() -> E,
    default_f: fn() -> F,
    default_t: fn() -> T,
}
crate::impl_index_vertex!(MwbComboMesh3<V, E, F, T>);
crate::impl_index_edge!(MwbComboMesh3<V, E, F, T>);
crate::impl_index_tri!(MwbComboMesh3<V, E, F, T>);
crate::impl_index_tet!(MwbComboMesh3<V, E, F, T>);
crate::impl_with_eft!(MwbComboMesh3<V, E, F, T>: <V, E, F, T> ComboMesh1<V, E>, <V, E, F, T> ComboMesh2<V, E, F>, <V, E, F, T> ComboMesh3<V, E, F, T>);

impl<V, E, F, T> HasVertices for MwbComboMesh3<V, E, F, T> {
    crate::impl_has_vertices!(HigherVertex<V> zeroed zeroed zeroed, Higher = B1);

    fn remove_vertex_higher<L: Lock>(&mut self, vertex: VertexId) {
        self.remove_edges(
            self.vertex_edges_out(vertex)
                .chain(self.vertex_edges_in(vertex))
                .collect::<Vec<_>>(),
        );
    }

    fn clear_vertices_higher<L: Lock>(&mut self) {
        self.tets.clear();
        self.tris.clear();
        self.edges.clear();
    }
}

impl<V, E, F, T> HasEdges for MwbComboMesh3<V, E, F, T> {
    crate::impl_has_edges!(HigherEdge<E> zeroed zeroed, Mwb = B0, Higher = B1);

    type WithoutEdges = ComboMesh0<V>;
    type WithMwbE = MwbComboMesh1<V, E>;
    type WithoutMwbE = ComboMesh1<V, E>;

    fn remove_edge_higher<L: Lock>(&mut self, edge: EdgeId) {
        // Preserve purity, and don't remove `edge` prematurely
        let mut opps = self.edge_vertex_opps(edge).collect::<Vec<_>>();
        if let Some(opp) = opps.first().copied() {
            self.remove_tris(
                opps.drain(1..)
                    .map(|v| TriId::from_valid([edge.0[0], edge.0[1], v])),
            );
            self.remove_tri_keep_edges(TriId::from_valid([edge.0[0], edge.0[1], opp]));

            // Edges don't have the mwb property here, so check if there are triangles around them
            if self.edge_vertex_opps(EdgeId([edge.0[1], opp])).count() == 0 {
                self.remove_edge(EdgeId([edge.0[1], opp]));
            }
            if self.edge_vertex_opps(EdgeId([opp, edge.0[0]])).count() == 0 {
                self.remove_edge(EdgeId([opp, edge.0[0]]));
            }
        }
    }

    fn clear_edges_higher<L: Lock>(&mut self) {
        self.tets.clear();
        self.tris.clear();
    }
}

impl<V, E, F, T> HasTris for MwbComboMesh3<V, E, F, T> {
    crate::impl_has_tris!(HigherTri<F> zeroed, Mwb = B0, Higher = B1);

    type WithoutTris = ComboMesh1<V, E>;
    type WithMwbF = MwbComboMesh2<V, E, F>;
    type WithoutMwbF = ComboMesh2<V, E, F>;

    fn remove_tri_higher<L: Lock>(&mut self, tri: TriId) {
        self.tri_vertex_opp(tri).map(|opp| {
            self.remove_tet_keep_tris(TetId::from_valid([tri.0[0], tri.0[1], tri.0[2], opp]));
            // Be careful not to remove `tri` as it will be removed after this function
            self.remove_tri(TriId::from_valid([opp, tri.0[2], tri.0[1]]));
            self.remove_tri(TriId::from_valid([tri.0[2], opp, tri.0[0]]));
            self.remove_tri(TriId::from_valid([tri.0[1], tri.0[0], opp]));
        });
    }

    fn clear_tris_higher<L: Lock>(&mut self) {
        self.tets.clear();
    }
}

impl<V, E, F, T> HasTets for MwbComboMesh3<V, E, F, T> {
    crate::impl_has_tets!(MwbTet<T>, Mwb = B1);

    type WithoutTets = ComboMesh2<V, E, F>;
    type WithMwbT = MwbComboMesh3<V, E, F, T>;
    type WithoutMwbT = ComboMesh3<V, E, F, T>;

    fn remove_tet_higher<L: Lock>(&mut self, _: TetId) {}

    fn clear_tets_higher<L: Lock>(&mut self) {}
}

impl<V: Default, E: Default, F: Default, T: Default> Default for MwbComboMesh3<V, E, F, T> {
    fn default() -> Self {
        MwbComboMesh3 {
            vertices: OrderedIdMap::default(),
            edges: FnvHashMap::default(),
            tris: FnvHashMap::default(),
            tets: FnvHashMap::default(),
            next_vertex_id: 0,
            default_v: Default::default,
            default_e: Default::default,
            default_f: Default::default,
            default_t: Default::default,
        }
    }
}

impl<V, E, F, T> MwbComboMesh3<V, E, F, T> {
    /// Creates an empty tet mesh.
    pub fn new() -> Self
    where
        V: Default,
        E: Default,
        F: Default,
        T: Default,
    {
        Self::default()
    }

    /// Creates an empty tet mesh with default values for elements.
    pub fn with_defaults(
        vertex: fn() -> V,
        edge: fn() -> E,
        tri: fn() -> F,
        tet: fn() -> T,
    ) -> Self {
        Self {
            vertices: OrderedIdMap::default(),
            edges: FnvHashMap::default(),
            tris: FnvHashMap::default(),
            tets: FnvHashMap::default(),
            next_vertex_id: 0,
            default_v: vertex,
            default_e: edge,
            default_f: tri,
            default_t: tet,
        }
    }
}

mod internal {
    use crate::edge::Link;
    use crate::vertex::VertexId;

    #[derive(Clone, Debug)]
    #[doc(hidden)]
    pub struct HigherTri<F> {
        /// Targets from the same edge for each of the edges,
        /// whether the triangle actually exists or not
        links: [Link<VertexId>; 3],
        tet_opp: VertexId,
        value: F,
    }
    #[rustfmt::skip]
    crate::impl_tri_higher!(
        HigherTri<F>,
        new |id, links, value| {
            HigherTri {
                tet_opp: id,
                links,
                value,
            }
        }
    );

    /// A tetrahedron of an tet mesh
    #[derive(Clone, Debug)]
    #[doc(hidden)]
    pub struct Tet<T> {
        /// Targets from the same triangle for each of the triangle,
        /// whether the tetrahedron actually exists or not
        links: [Link<VertexId>; 4],
        value: T,
    }
    #[rustfmt::skip]
    crate::impl_tet!(Tet<T>, new |_id, links, value| Tet { links, value });

    /// A tetrahedron of a mwb tet mesh.
    #[derive(Clone, Debug)]
    #[doc(hidden)]
    pub struct MwbTet<T> {
        value: T,
    }
    crate::impl_tet_mwb!(MwbTet<T>, new | _id, _links, value | MwbTet { value });
}

#[cfg(test)]
mod tests {
    use super::*;
    use fnv::FnvHashSet;
    use std::convert::TryInto;
    use std::fmt::Debug;
    use std::hash::Hash;

    #[track_caller]
    fn assert_vertices<
        V: Clone + Debug + Eq + Hash,
        E,
        F,
        T,
        I: IntoIterator<Item = (VertexId, V)>,
    >(
        mesh: &ComboMesh3<V, E, F, T>,
        vertices: I,
    ) {
        let result = mesh
            .vertices()
            .map(|(id, v)| (*id, v.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = vertices.into_iter().collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
    }

    #[track_caller]
    fn assert_edges<
        V,
        E: Clone + Debug + Eq + Hash,
        EI: TryInto<EdgeId>,
        F,
        T,
        I: IntoIterator<Item = (EI, E)>,
    >(
        mesh: &ComboMesh3<V, E, F, T>,
        edges: I,
    ) {
        let result = mesh
            .edges()
            .map(|(id, e)| (*id, e.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = edges
            .into_iter()
            .map(|(vertices, e)| (vertices.try_into().ok().unwrap(), e))
            .collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
        assert_eq!(mesh.num_edges(), expect.len());
    }

    #[track_caller]
    fn assert_tris<
        V,
        E,
        F: Clone + Debug + Eq + Hash,
        FI: TryInto<TriId>,
        T,
        I: IntoIterator<Item = (FI, F)>,
    >(
        mesh: &ComboMesh3<V, E, F, T>,
        tris: I,
    ) {
        let result = mesh
            .tris()
            .map(|(id, f)| (*id, f.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = tris
            .into_iter()
            .map(|(vertices, f)| (vertices.try_into().ok().unwrap(), f))
            .collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
        assert_eq!(mesh.num_tris(), expect.len());
    }

    #[track_caller]
    fn assert_tets<
        V,
        E,
        F,
        T: Clone + Debug + Eq + Hash,
        TI: TryInto<TetId>,
        I: IntoIterator<Item = (TI, T)>,
    >(
        mesh: &ComboMesh3<V, E, F, T>,
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

    #[track_caller]
    fn assert_vertices_m<
        V: Clone + Debug + Eq + Hash,
        E,
        F,
        T,
        I: IntoIterator<Item = (VertexId, V)>,
    >(
        mesh: &MwbComboMesh3<V, E, F, T>,
        vertices: I,
    ) {
        let result = mesh
            .vertices()
            .map(|(id, v)| (*id, v.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = vertices.into_iter().collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
    }

    #[track_caller]
    fn assert_edges_m<
        V,
        E: Clone + Debug + Eq + Hash,
        EI: TryInto<EdgeId>,
        F,
        T,
        I: IntoIterator<Item = (EI, E)>,
    >(
        mesh: &MwbComboMesh3<V, E, F, T>,
        edges: I,
    ) {
        let result = mesh
            .edges()
            .map(|(id, e)| (*id, e.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = edges
            .into_iter()
            .map(|(vertices, e)| (vertices.try_into().ok().unwrap(), e))
            .collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
        assert_eq!(mesh.num_edges(), expect.len());
    }

    #[track_caller]
    fn assert_tris_m<
        V,
        E,
        F: Clone + Debug + Eq + Hash,
        FI: TryInto<TriId>,
        T,
        I: IntoIterator<Item = (FI, F)>,
    >(
        mesh: &MwbComboMesh3<V, E, F, T>,
        tris: I,
    ) {
        let result = mesh
            .tris()
            .map(|(id, f)| (*id, f.clone()))
            .collect::<FnvHashSet<_>>();
        let expect = tris
            .into_iter()
            .map(|(vertices, f)| (vertices.try_into().ok().unwrap(), f))
            .collect::<FnvHashSet<_>>();

        assert_eq!(result, expect);
        assert_eq!(mesh.num_tris(), expect.len());
    }

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
    fn test_default() {
        let mesh = ComboMesh3::<(), (), (), ()>::default();
        assert!(mesh.vertices.is_empty());
        assert!(mesh.edges.is_empty());
        assert!(mesh.tris.is_empty());
        assert!(mesh.tets.is_empty());
        assert_eq!(mesh.num_edges(), 0);
        assert_eq!(mesh.num_tris(), 0);
        assert_eq!(mesh.num_tets(), 0);
    }

    #[test]
    fn test_add_tet() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2]);
        assert_eq!(mesh.add_tet([ids[1], ids[0], ids[2], ids[3]], 1), None);
        assert_tris(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
            ],
        );
        assert_tets(&mesh, vec![([ids[0], ids[1], ids[3], ids[2]], 1)]);

        // Prematurely add triangle
        mesh.add_tri([ids[1], ids[2], ids[0]], 1);

        // Add twin
        assert_eq!(mesh.add_tet([ids[1], ids[0], ids[3], ids[2]], 2), None);
        assert_tris(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
                ([ids[0], ids[1], ids[2]], 1),
                ([ids[3], ids[2], ids[1]], 0),
                ([ids[2], ids[3], ids[0]], 0),
                ([ids[1], ids[0], ids[3]], 0),
            ],
        );
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[3], ids[2]], 1),
                ([ids[0], ids[1], ids[2], ids[3]], 2),
            ],
        );

        // Modify tet
        assert_eq!(mesh.add_tet([ids[3], ids[2], ids[1], ids[0]], 3), Some(2));
        assert_tris(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
                ([ids[0], ids[1], ids[2]], 1),
                ([ids[3], ids[2], ids[1]], 0),
                ([ids[2], ids[3], ids[0]], 0),
                ([ids[1], ids[0], ids[3]], 0),
            ],
        );
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[3], ids[2]], 1),
                ([ids[0], ids[1], ids[2], ids[3]], 3),
            ],
        );
    }

    #[test]
    fn test_extend_tets() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);

        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        assert_eq!(mesh.num_tris(), 23);
        assert_tets(&mesh, tets);
    }

    #[test]
    fn test_remove_vertex() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_vertex(ids[1]), Some(6)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 30);
        assert_eq!(mesh.num_tris(), 17);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_vertex(ids[3]), Some(2)); // Multiple tets should be removed
        assert_eq!(mesh.num_edges(), 20);
        assert_eq!(mesh.num_tris(), 7);
        assert_tets(&mesh, vec![([ids[6], ids[7], ids[4], ids[5]], 6)]);
    }

    #[test]
    fn test_remove_edge() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_edge([ids[1], ids[0]]), Some(0)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 35);
        assert_eq!(mesh.num_tris(), 19);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_edge([ids[2], ids[3]]), Some(0)); // Multiple tets should be removed
        assert_eq!(mesh.num_edges(), 34);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets(
            &mesh,
            vec![
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_tri() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tri([ids[2], ids[1], ids[0]]), Some(0)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 34);
        assert_eq!(mesh.num_tris(), 19);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_add_vertex() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2]);
        let edges = vec![
            ([ids[0], ids[3]], 5),
            ([ids[1], ids[3]], 3),
            ([ids[3], ids[1]], 2),
            ([ids[2], ids[3]], 8),
            ([ids[1], ids[2]], 9),
        ];
        mesh.extend_edges(edges.clone());

        mesh.remove_vertex(ids[1]);
        assert_vertices(&mesh, vec![(ids[0], 3), (ids[2], 9), (ids[3], 2)]);
        assert_edges(&mesh, vec![([ids[0], ids[3]], 5), ([ids[2], ids[3]], 8)]);

        let id2 = mesh.add_vertex(6);
        assert_vertices(&mesh, vec![(ids[0], 3), (ids[2], 9), (ids[3], 2), (id2, 6)]);
        assert_edges(&mesh, vec![([ids[0], ids[3]], 5), ([ids[2], ids[3]], 8)]);
    }

    #[test]
    fn test_remove_tet() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tet([ids[0], ids[1], ids[2], ids[3]]), Some(1));
        assert_eq!(mesh.num_tris(), 20);
        assert_tets(
            &mesh,
            vec![
                ([ids[1], ids[2], ids[3], ids[0]], 2),
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_tet([ids[0], ids[1], ids[3], ids[2]]), Some(2));
        assert_eq!(mesh.num_tris(), 16);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_tet([ids[4], ids[1], ids[3], ids[2]]), None);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_add_tet() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tet([ids[0], ids[1], ids[2], ids[3]]), Some(1));
        assert_eq!(mesh.add_tet([ids[7], ids[6], ids[4], ids[8]], 7), None);
        assert_eq!(mesh.add_tet([ids[0], ids[1], ids[2], ids[3]], 8), None);
        assert_eq!(mesh.num_tris(), 27);
        assert_tets(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[2], ids[3]], 8),
                ([ids[1], ids[2], ids[3], ids[0]], 2),
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
                ([ids[7], ids[6], ids[4], ids[8]], 7),
            ],
        );
    }

    #[test]
    fn test_remove_add_edge() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2]);
        let edges = vec![
            ([ids[0], ids[3]], 5),
            ([ids[1], ids[3]], 3),
            ([ids[3], ids[1]], 2),
            ([ids[2], ids[3]], 8),
            ([ids[1], ids[2]], 9),
        ];
        mesh.extend_edges(edges.clone());

        mesh.remove_edge([ids[1], ids[3]]); // first outgoing edge from vertex
        assert_edges(
            &mesh,
            vec![
                ([ids[0], ids[3]], 5),
                ([ids[3], ids[1]], 2),
                ([ids[2], ids[3]], 8),
                ([ids[1], ids[2]], 9),
            ],
        );

        mesh.add_edge([ids[1], ids[0]], 4);
        mesh.add_edge([ids[1], ids[3]], 6);
        assert_edges(
            &mesh,
            vec![
                ([ids[0], ids[3]], 5),
                ([ids[3], ids[1]], 2),
                ([ids[2], ids[3]], 8),
                ([ids[1], ids[2]], 9),
                ([ids[1], ids[0]], 4),
                ([ids[1], ids[3]], 6),
            ],
        );
    }

    #[test]
    fn test_clear_vertices() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let edges = vec![
            ([ids[0], ids[3]], 5),
            ([ids[1], ids[3]], 3),
            ([ids[3], ids[1]], 2),
            ([ids[2], ids[3]], 8),
            ([ids[1], ids[2]], 9),
        ];
        mesh.extend_edges(edges.clone());

        mesh.clear_vertices();
        assert_vertices(&mesh, vec![]);
        assert_edges(&mesh, vec![] as Vec<(EdgeId, _)>);
        assert_tris(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_edges() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tris = vec![
            ([ids[0], ids[1], ids[2]], 1),
            ([ids[3], ids[1], ids[2]], 2),
            ([ids[4], ids[2], ids[1]], 3),
            ([ids[4], ids[1], ids[5]], 4),
            ([ids[5], ids[6], ids[4]], 5),
            ([ids[4], ids[6], ids[5]], 6),
        ];
        mesh.extend_tris(tris);

        mesh.clear_edges();
        assert_vertices(
            &mesh,
            vec![
                (ids[0], 3),
                (ids[1], 6),
                (ids[2], 9),
                (ids[3], 2),
                (ids[4], 5),
                (ids[5], 8),
                (ids[6], 1),
                (ids[7], 4),
            ],
        );
        assert_edges(&mesh, vec![] as Vec<(EdgeId, _)>);
        assert_tris(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_tris() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tris = vec![
            ([ids[0], ids[1], ids[2]], 1),
            ([ids[3], ids[1], ids[2]], 2),
            ([ids[4], ids[2], ids[1]], 3),
            ([ids[4], ids[1], ids[5]], 4),
            ([ids[5], ids[6], ids[4]], 5),
            ([ids[4], ids[6], ids[5]], 6),
        ];
        mesh.extend_tris(tris);

        mesh.clear_tris();
        assert_vertices(
            &mesh,
            vec![
                (ids[0], 3),
                (ids[1], 6),
                (ids[2], 9),
                (ids[3], 2),
                (ids[4], 5),
                (ids[5], 8),
                (ids[6], 1),
                (ids[7], 4),
            ],
        );
        assert_edges(
            &mesh,
            vec![
                ([ids[0], ids[1]], 0),
                ([ids[1], ids[2]], 0),
                ([ids[2], ids[0]], 0),
                ([ids[3], ids[1]], 0),
                ([ids[2], ids[3]], 0),
                ([ids[4], ids[2]], 0),
                ([ids[2], ids[1]], 0),
                ([ids[1], ids[4]], 0),
                ([ids[4], ids[1]], 0),
                ([ids[1], ids[5]], 0),
                ([ids[5], ids[4]], 0),
                ([ids[5], ids[6]], 0),
                ([ids[6], ids[4]], 0),
                ([ids[4], ids[5]], 0),
                ([ids[4], ids[6]], 0),
                ([ids[6], ids[5]], 0),
            ],
        );
        assert_tris(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_tets() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        mesh.clear_tets();
        assert_eq!(mesh.num_vertices(), 9);
        assert_ne!(mesh.num_edges(), 0); // Don't want to think about this number
        assert_eq!(mesh.num_tris(), 23);
        assert_tets(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_tet_walker() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        mesh.add_tri([ids[6], ids[7], ids[8]], 7);

        assert!(mesh.tet_walker_from_tri([ids[6], ids[7], ids[8]]).is_none());

        let walker = mesh.tet_walker_from_edge_edge([ids[2], ids[3]], [ids[0], ids[1]]);
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[1]]));
        assert_eq!(walker.first(), ids[2]);
        assert_eq!(walker.second(), ids[3]);
        assert_eq!(walker.third(), ids[0]);
        assert_eq!(walker.fourth(), ids[1]);
        assert_eq!(
            walker.tri(),
            [ids[2], ids[3], ids[0]].try_into().ok().unwrap()
        );
        assert_eq!(
            walker.tet(),
            [ids[2], ids[3], ids[0], ids[1]].try_into().ok().unwrap()
        );

        let branch = walker.twin().unwrap(); // branch!
        assert_eq!(branch.edge(), EdgeId([ids[3], ids[2]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[0], ids[1]]));

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[0]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[1]]));

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[0], ids[2]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[3], ids[1]]));

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[1]]));

        let branch = walker.prev_edge(); // branch!
        assert_eq!(branch.edge(), EdgeId([ids[0], ids[2]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[3], ids[1]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[1], ids[0]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[3], ids[2]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[0], ids[1]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[3]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[2]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[1]]));

        let branch = walker.prev_tri(); // branch!
        assert_eq!(branch.edge(), EdgeId([ids[3], ids[2]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[1], ids[0]]));

        let branch = branch.flip_tri(); // branch continue!
        assert_eq!(branch.edge(), EdgeId([ids[1], ids[0]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[3], ids[2]]));

        let walker = walker.next_opp();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[4]]));

        let walker = walker.prev_opp();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[1]]));

        let walker = mesh.tet_walker_from_edge_edge([ids[4], ids[3]], [ids[2], ids[0]]);
        let walker = walker.on_twin_tri().unwrap();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[4]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[5]]));

        let walker = walker.prev_tri().on_twin_tri().unwrap();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[4]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[5], ids[6]]));

        assert!(walker.twin().is_none());
    }

    #[test]
    fn test_vertex_edges_out() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5]);
        let edges = vec![
            ([ids[0], ids[3]], 5),
            ([ids[1], ids[3]], 3),
            ([ids[3], ids[1]], 2),
            ([ids[1], ids[2]], 9),
            ([ids[2], ids[3]], 8),
        ];
        mesh.extend_edges(edges.clone());

        let set = mesh.vertex_edges_out(ids[4]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_edges_out(ids[2]).collect::<FnvHashSet<_>>();
        let expected = vec![EdgeId([ids[2], ids[3]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_edges_out(ids[1]).collect::<FnvHashSet<_>>();
        let expected = vec![EdgeId([ids[1], ids[3]]), EdgeId([ids[1], ids[2]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_vertex_edges_in() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5]);
        let edges = vec![
            ([ids[0], ids[3]], 5),
            ([ids[1], ids[3]], 3),
            ([ids[3], ids[1]], 2),
            ([ids[1], ids[2]], 9),
        ];
        mesh.extend_edges(edges.clone());

        let set = mesh.vertex_edges_in(ids[4]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_edges_in(ids[2]).collect::<FnvHashSet<_>>();
        let expected = vec![EdgeId([ids[1], ids[2]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_edges_in(ids[3]).collect::<FnvHashSet<_>>();
        let expected = vec![EdgeId([ids[0], ids[3]]), EdgeId([ids[1], ids[3]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_edge_tris() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tris = vec![
            ([ids[0], ids[1], ids[2]], 1),
            ([ids[3], ids[1], ids[2]], 2),
            ([ids[4], ids[2], ids[1]], 3),
            ([ids[4], ids[1], ids[5]], 4),
            ([ids[5], ids[6], ids[4]], 5),
            ([ids[4], ids[6], ids[5]], 6),
        ];
        mesh.extend_tris(tris);
        mesh.add_edge([ids[6], ids[7]], 1);

        let set = mesh.edge_tris([ids[6], ids[7]]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.edge_tris([ids[0], ids[1]]).collect::<FnvHashSet<_>>();
        let expected = vec![TriId([ids[0], ids[1], ids[2]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.edge_tris([ids[1], ids[2]]).collect::<FnvHashSet<_>>();
        let expected = vec![
            TriId([ids[0], ids[1], ids[2]]),
            TriId([ids[1], ids[2], ids[3]]),
        ]
        .into_iter()
        .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_vertex_tris() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tris = vec![
            ([ids[0], ids[1], ids[2]], 1),
            ([ids[3], ids[1], ids[2]], 2),
            ([ids[4], ids[2], ids[1]], 3),
            ([ids[4], ids[1], ids[5]], 4),
            ([ids[5], ids[6], ids[4]], 5),
        ];
        mesh.extend_tris(tris);
        mesh.add_edge([ids[6], ids[7]], 1);

        let set = mesh.vertex_tris(ids[7]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_tris(ids[6]).collect::<FnvHashSet<_>>();
        let expected = vec![TriId([ids[4], ids[5], ids[6]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh.vertex_tris(ids[1]).collect::<FnvHashSet<_>>();
        let expected = vec![
            TriId([ids[0], ids[1], ids[2]]),
            TriId([ids[1], ids[2], ids[3]]),
            TriId([ids[1], ids[4], ids[2]]),
            TriId([ids[1], ids[5], ids[4]]),
        ]
        .into_iter()
        .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_tri_tets() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        mesh.add_tri([ids[6], ids[7], ids[8]], 7);

        let set = mesh
            .tri_tets([ids[6], ids[7], ids[8]])
            .collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh
            .tri_tets([ids[4], ids[5], ids[6]])
            .collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let set = mesh
            .tri_tets([ids[0], ids[2], ids[3]])
            .collect::<FnvHashSet<_>>();
        let expected = vec![
            TetId([ids[0], ids[1], ids[2], ids[3]]),
            TetId([ids[0], ids[2], ids[3], ids[4]]),
        ]
        .into_iter()
        .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_edge_tets() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        mesh.add_tri([ids[6], ids[7], ids[8]], 7);

        let set = mesh.edge_tets([ids[7], ids[8]]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let list = mesh.edge_tets([ids[6], ids[7]]).collect::<Vec<_>>();
        assert_eq!(list.len(), 1);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let list = mesh.edge_tets([ids[3], ids[4]]).collect::<Vec<_>>();
        assert_eq!(list.len(), 3);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![
            TetId([ids[0], ids[2], ids[3], ids[4]]),
            TetId([ids[2], ids[3], ids[4], ids[5]]),
            TetId([ids[3], ids[4], ids[5], ids[6]]),
        ]
        .into_iter()
        .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_vertex_tets() {
        let mut mesh = ComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1),
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        mesh.add_tri([ids[6], ids[7], ids[8]], 7);

        let set = mesh.vertex_tets(ids[8]).collect::<FnvHashSet<_>>();
        let expected = vec![].into_iter().collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let list = mesh.vertex_tets(ids[7]).collect::<Vec<_>>();
        assert_eq!(list.len(), 1);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);

        let list = mesh.vertex_tets(ids[4]).collect::<Vec<_>>();
        assert_eq!(list.len(), 4);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![
            TetId([ids[0], ids[2], ids[3], ids[4]]),
            TetId([ids[2], ids[3], ids[4], ids[5]]),
            TetId([ids[3], ids[4], ids[5], ids[6]]),
            TetId([ids[4], ids[5], ids[6], ids[7]]),
        ]
        .into_iter()
        .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_default_m() {
        let mesh = MwbComboMesh3::<(), (), (), ()>::default();
        assert!(mesh.vertices.is_empty());
        assert!(mesh.edges.is_empty());
        assert!(mesh.tris.is_empty());
        assert!(mesh.tets.is_empty());
        assert_eq!(mesh.num_edges(), 0);
        assert_eq!(mesh.num_tris(), 0);
        assert_eq!(mesh.num_tets(), 0);
    }

    #[test]
    fn test_add_tet_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2]);
        assert_eq!(mesh.add_tet([ids[1], ids[0], ids[2], ids[3]], 1), None);
        assert_tris_m(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
            ],
        );
        assert_tets_m(&mesh, vec![([ids[0], ids[1], ids[3], ids[2]], 1)]);

        // Add twin
        assert_eq!(mesh.add_tet([ids[1], ids[0], ids[3], ids[2]], 2), None);
        assert_tris_m(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
                ([ids[0], ids[1], ids[2]], 0),
                ([ids[3], ids[2], ids[1]], 0),
                ([ids[2], ids[3], ids[0]], 0),
                ([ids[1], ids[0], ids[3]], 0),
            ],
        );
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[3], ids[2]], 1),
                ([ids[0], ids[1], ids[2], ids[3]], 2),
            ],
        );

        // Modify tet
        assert_eq!(mesh.add_tet([ids[3], ids[2], ids[1], ids[0]], 3), Some(2));
        assert_tris_m(
            &mesh,
            vec![
                ([ids[2], ids[1], ids[0]], 0),
                ([ids[1], ids[2], ids[3]], 0),
                ([ids[0], ids[3], ids[2]], 0),
                ([ids[3], ids[0], ids[1]], 0),
                ([ids[0], ids[1], ids[2]], 0),
                ([ids[3], ids[2], ids[1]], 0),
                ([ids[2], ids[3], ids[0]], 0),
                ([ids[1], ids[0], ids[3]], 0),
            ],
        );
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[3], ids[2]], 1),
                ([ids[0], ids[1], ids[2], ids[3]], 3),
            ],
        );
    }

    #[test]
    fn test_extend_tets_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);

        let tets = vec![
            ([ids[0], ids[1], ids[2], ids[3]], 1), // killed by 0-2-3-4
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());
        assert_eq!(mesh.num_tris(), 20);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[1], ids[2], ids[3], ids[0]], 2),
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_vertex_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_vertex(ids[1]), Some(6)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 30);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_vertex(ids[3]), Some(2)); // Multiple tets should be removed
        assert_eq!(mesh.num_edges(), 12);
        assert_eq!(mesh.num_tris(), 4);
        assert_tets_m(&mesh, vec![([ids[6], ids[7], ids[4], ids[5]], 6)]);
    }

    #[test]
    fn test_remove_edge_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_edge([ids[1], ids[0]]), Some(0)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 30);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_edge([ids[2], ids[3]]), Some(0)); // Multiple tets should be removed
        assert_eq!(mesh.num_edges(), 18);
        assert_eq!(mesh.num_tris(), 8);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_tri_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tri([ids[2], ids[1], ids[0]]), Some(0)); // Only 1 tet should be removed
        assert_eq!(mesh.num_edges(), 30);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_tet_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tet([ids[0], ids[1], ids[3], ids[2]]), Some(2));
        assert_eq!(mesh.num_tris(), 16);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );

        assert_eq!(mesh.remove_tet([ids[4], ids[1], ids[3], ids[2]]), None);
        assert_eq!(mesh.num_tris(), 16);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[2], ids[3], ids[4]], 3),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
            ],
        );
    }

    #[test]
    fn test_remove_add_tet_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        assert_eq!(mesh.remove_tet([ids[0], ids[1], ids[3], ids[2]]), Some(2));
        assert_eq!(mesh.add_tet([ids[7], ids[6], ids[4], ids[8]], 7), None);
        assert_eq!(mesh.add_tet([ids[0], ids[1], ids[2], ids[3]], 8), None);
        assert_eq!(mesh.num_tris(), 20);
        assert_tets_m(
            &mesh,
            vec![
                ([ids[0], ids[1], ids[2], ids[3]], 8),
                ([ids[2], ids[3], ids[4], ids[5]], 4),
                ([ids[6], ids[5], ids[4], ids[3]], 5),
                ([ids[6], ids[7], ids[4], ids[5]], 6),
                ([ids[7], ids[6], ids[4], ids[8]], 7),
            ],
        );
    }

    #[test]
    fn test_clear_vertices_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        mesh.clear_vertices();
        assert_vertices_m(&mesh, vec![]);
        assert_edges_m(&mesh, vec![] as Vec<(EdgeId, _)>);
        assert_tris_m(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets_m(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_edges_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets);

        mesh.clear_edges();
        assert_vertices_m(
            &mesh,
            vec![
                (ids[0], 3),
                (ids[1], 6),
                (ids[2], 9),
                (ids[3], 2),
                (ids[4], 5),
                (ids[5], 8),
                (ids[6], 1),
                (ids[7], 4),
            ],
        );
        assert_edges_m(&mesh, vec![] as Vec<(EdgeId, _)>);
        assert_tris_m(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets_m(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_tris_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets);

        mesh.clear_tris();
        assert_vertices_m(
            &mesh,
            vec![
                (ids[0], 3),
                (ids[1], 6),
                (ids[2], 9),
                (ids[3], 2),
                (ids[4], 5),
                (ids[5], 8),
                (ids[6], 1),
                (ids[7], 4),
            ],
        );
        assert_ne!(mesh.num_edges(), 0); // Don't want to think about this number
        assert_tris_m(&mesh, vec![] as Vec<(TriId, _)>);
        assert_tets_m(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_clear_tets_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        mesh.clear_tets();
        assert_eq!(mesh.num_vertices(), 9);
        assert_ne!(mesh.num_edges(), 0); // Don't want to think about this number
        assert_eq!(mesh.num_tris(), 20);
        assert_tets_m(&mesh, vec![] as Vec<(TetId, _)>);
    }

    #[test]
    fn test_tet_walker_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        let walker = mesh.tet_walker_from_edge_edge([ids[2], ids[3]], [ids[1], ids[0]]);
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));
        assert_eq!(walker.first(), ids[2]);
        assert_eq!(walker.second(), ids[3]);
        assert_eq!(walker.third(), ids[1]);
        assert_eq!(walker.fourth(), ids[0]);
        assert_eq!(
            walker.tri(),
            [ids[2], ids[3], ids[1]].try_into().ok().unwrap()
        );
        assert_eq!(
            walker.tet(),
            [ids[2], ids[3], ids[1], ids[0]].try_into().ok().unwrap()
        );

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[1]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[0]]));

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[1], ids[2]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[3], ids[0]]));

        let walker = walker.next_edge();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));

        let branch = walker.prev_edge(); // branch!
        assert_eq!(branch.edge(), EdgeId([ids[1], ids[2]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[3], ids[0]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[2]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[0], ids[1]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[1], ids[0]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[3]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[0], ids[1]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[3], ids[2]]));

        let walker = walker.next_tri();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));

        let branch = walker.prev_tri(); // branch!
        assert_eq!(branch.edge(), EdgeId([ids[0], ids[1]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[3], ids[2]]));

        let branch = branch.flip_tri(); // branch continue!
        assert_eq!(branch.edge(), EdgeId([ids[3], ids[2]]));
        assert_eq!(branch.opp_edge(), EdgeId([ids[0], ids[1]]));

        let walker = walker.next_opp();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));

        let walker = walker.prev_opp();
        assert_eq!(walker.edge(), EdgeId([ids[2], ids[3]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[1], ids[0]]));

        let walker = mesh.tet_walker_from_edge_edge([ids[4], ids[3]], [ids[2], ids[0]]);
        let walker = walker.on_twin_tri().unwrap();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[4]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[2], ids[5]]));

        let walker = walker.prev_tri().on_twin_tri().unwrap();
        assert_eq!(walker.edge(), EdgeId([ids[3], ids[4]]));
        assert_eq!(walker.opp_edge(), EdgeId([ids[5], ids[6]]));

        assert!(walker.twin().is_none());
    }

    #[test]
    fn test_tri_tets_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        let set = mesh
            .tri_tets([ids[4], ids[5], ids[6]])
            .collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_edge_tets_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        let list = mesh.edge_tets([ids[6], ids[7]]).collect::<Vec<_>>();
        assert_eq!(list.len(), 1);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }

    #[test]
    fn test_vertex_tets_m() {
        let mut mesh = MwbComboMesh3::<usize, usize, usize, usize>::default();
        let ids = mesh.extend_vertices(vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);
        let tets = vec![
            ([ids[1], ids[2], ids[3], ids[0]], 2),
            ([ids[0], ids[2], ids[3], ids[4]], 3),
            ([ids[2], ids[3], ids[4], ids[5]], 4),
            ([ids[6], ids[5], ids[4], ids[3]], 5),
            ([ids[6], ids[7], ids[4], ids[5]], 6),
        ];
        mesh.extend_tets(tets.clone());

        let list = mesh.vertex_tets(ids[7]).collect::<Vec<_>>();
        assert_eq!(list.len(), 1);
        let set = list.into_iter().collect::<FnvHashSet<_>>();
        let expected = vec![TetId([ids[4], ids[5], ids[6], ids[7]])]
            .into_iter()
            .collect::<FnvHashSet<_>>();
        assert_eq!(set, expected);
    }
}

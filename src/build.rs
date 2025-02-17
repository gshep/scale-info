// Copyright 2019-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Builders for defining metadata for variant types (enums), and composite types (structs).
//! They are designed to allow only construction of valid definitions.
//!
//! In most cases we recommend using the `scale-info-derive` crate to auto generate the builder
//! constructions.
//!
//! # Examples
//!
//! ## Generic struct
//! ```
//! # use scale_info::{build::Fields, type_params, MetaType, Path, Type, TypeInfo};
//! struct Foo<T> {
//!     bar: T,
//!     data: u64,
//! }
//!
//! impl<T> TypeInfo for Foo<T>
//! where
//!     T: TypeInfo + 'static,
//! {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .type_params(type_params!(T))
//!             .composite(Fields::named()
//!                 .field(|f| f.ty::<T>().name("bar").type_name("T"))
//!                 .field(|f| f.ty::<u64>().name("data").type_name("u64"))
//!             )
//!     }
//! }
//! ```
//! ## Tuple struct
//! ```
//! # use scale_info::{build::Fields, MetaType, Path, Type, TypeInfo};
//! struct Foo(u32, bool);
//!
//! impl TypeInfo for Foo {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .composite(Fields::unnamed()
//!                 .field(|f| f.ty::<u32>().type_name("u32"))
//!                 .field(|f| f.ty::<bool>().type_name("bool"))
//!             )
//!     }
//! }
//! ```
//! ## Enum with fields
//! ```
//! # use scale_info::{build::{Fields, Variants}, type_params, MetaType, Path, Type, TypeInfo, Variant};
//! enum Foo<T>{
//!     A(T),
//!     B { f: u32 },
//!     C,
//! }
//!
//! impl<T> TypeInfo for Foo<T>
//! where
//!     T: TypeInfo + 'static,
//! {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!                .type_params(type_params!(T))
//!             .variant(
//!                 Variants::new()
//!                     .variant("A", |v| v
//!                         .index(0)
//!                         .fields(Fields::unnamed().field(|f| f.ty::<T>().type_name("T")))
//!                     )
//!                     .variant("B", |v| v
//!                         .index(1)
//!                         .fields(Fields::named().field(|f| f.ty::<u32>().name("f").type_name("u32")))
//!                     )
//!                     .variant_unit("A", 2)
//!             )
//!     }
//! }
//! ```
//! ## Enum without fields, aka C-style enums.
//! ```
//! # use scale_info::{build::{Fields, Variants}, MetaType, Path, Type, TypeInfo, Variant};
//! enum Foo {
//!     A,
//!     B,
//!     C = 33,
//! }
//!
//! impl TypeInfo for Foo {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .variant(
//!                 Variants::new()
//!                     .variant("A", |v| v.index(1))
//!                     .variant("B", |v| v.index(2))
//!                     .variant("C", |v| v.index(33))
//!             )
//!     }
//! }
//! ```

use crate::prelude::{
    marker::PhantomData,
    vec::Vec,
};

use crate::{
    form::MetaForm,
    Field,
    MetaType,
    Path,
    Type,
    TypeDef,
    TypeDefComposite,
    TypeDefVariant,
    TypeInfo,
    TypeParameter,
    Variant,
};

/// State types for type builders which require a Path.
pub mod state {
    /// State where the builder has not assigned a Path to the type
    pub enum PathNotAssigned {}
    /// State where the builder has assigned a Path to the type
    pub enum PathAssigned {}
}

/// Builds a [`Type`](`crate::Type`)
#[must_use]
pub struct TypeBuilder<S = state::PathNotAssigned> {
    path: Option<Path>,
    type_params: Vec<TypeParameter>,
    docs: Vec<&'static str>,
    marker: PhantomData<fn() -> S>,
}

impl<S> Default for TypeBuilder<S> {
    fn default() -> Self {
        TypeBuilder {
            path: Default::default(),
            type_params: Default::default(),
            docs: Default::default(),
            marker: Default::default(),
        }
    }
}

impl TypeBuilder<state::PathNotAssigned> {
    /// Set the Path for the type
    pub fn path(self, path: Path) -> TypeBuilder<state::PathAssigned> {
        TypeBuilder {
            path: Some(path),
            type_params: self.type_params,
            docs: self.docs,
            marker: Default::default(),
        }
    }
}

impl TypeBuilder<state::PathAssigned> {
    fn build<D>(self, type_def: D) -> Type
    where
        D: Into<TypeDef>,
    {
        let path = self.path.expect("Path not assigned");
        Type::new(path, self.type_params, type_def, self.docs)
    }

    /// Construct a "variant" type i.e an `enum`
    pub fn variant(self, builder: Variants) -> Type {
        self.build(builder.finalize())
    }

    /// Construct a "composite" type i.e. a `struct`
    pub fn composite<F>(self, fields: FieldsBuilder<F>) -> Type {
        self.build(TypeDefComposite::new(fields.finalize()))
    }
}

impl<S> TypeBuilder<S> {
    /// Set the type parameters if it's a generic type
    pub fn type_params<I>(mut self, type_params: I) -> Self
    where
        I: IntoIterator<Item = TypeParameter>,
    {
        self.type_params = type_params.into_iter().collect();
        self
    }

    #[cfg(feature = "docs")]
    /// Set the type documentation
    pub fn docs(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }

    #[cfg(not(feature = "docs"))]
    #[inline]
    /// Doc capture is not enabled via the "docs" feature so this is a no-op.
    pub fn docs(self, _docs: &'static [&'static str]) -> Self {
        self
    }

    /// Set the type documentation, always captured even if the "docs" feature is not enabled.
    pub fn docs_always(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }
}

/// A fields builder has no fields (e.g. a unit struct)
pub enum NoFields {}
/// A fields builder only allows named fields (e.g. a struct)
pub enum NamedFields {}
/// A fields builder only allows unnamed fields (e.g. a tuple)
pub enum UnnamedFields {}

/// Provides FieldsBuilder constructors
pub enum Fields {}

impl Fields {
    /// The type construct has no fields
    pub fn unit() -> FieldsBuilder<NoFields> {
        FieldsBuilder::<NoFields>::default()
    }

    /// Fields for a type construct with named fields
    pub fn named() -> FieldsBuilder<NamedFields> {
        FieldsBuilder::default()
    }

    /// Fields for a type construct with unnamed fields
    pub fn unnamed() -> FieldsBuilder<UnnamedFields> {
        FieldsBuilder::default()
    }
}

/// Build a set of either all named (e.g. for a struct) or all unnamed (e.g. for a tuple struct)
#[must_use]
pub struct FieldsBuilder<T> {
    fields: Vec<Field>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for FieldsBuilder<T> {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            marker: Default::default(),
        }
    }
}

impl<T> FieldsBuilder<T> {
    /// Complete building and return the set of fields
    pub fn finalize(self) -> Vec<Field<MetaForm>> {
        self.fields
    }

    fn push_field(mut self, field: Field) -> Self {
        // filter out fields of PhantomData
        if !field.ty().is_phantom() {
            self.fields.push(field);
        }
        self
    }
}

impl FieldsBuilder<NamedFields> {
    /// Add a named field constructed using the builder.
    pub fn field<F>(self, builder: F) -> Self
    where
        F: Fn(
            FieldBuilder,
        )
            -> FieldBuilder<field_state::NameAssigned, field_state::TypeAssigned>,
    {
        let builder = builder(FieldBuilder::new());
        self.push_field(builder.finalize())
    }
}

impl FieldsBuilder<UnnamedFields> {
    /// Add an unnamed field constructed using the builder.
    pub fn field<F>(self, builder: F) -> Self
    where
        F: Fn(
            FieldBuilder,
        )
            -> FieldBuilder<field_state::NameNotAssigned, field_state::TypeAssigned>,
    {
        let builder = builder(FieldBuilder::new());
        self.push_field(builder.finalize())
    }
}

/// Type states for building a field.
pub mod field_state {
    /// A name has not been assigned to the field.
    pub enum NameNotAssigned {}
    /// A name has been assigned to the field.
    pub enum NameAssigned {}
    /// A type has not been assigned to the field.
    pub enum TypeNotAssigned {}
    /// A type has been assigned to the field.
    pub enum TypeAssigned {}
}

/// Construct a valid [`Field`].
#[must_use]
pub struct FieldBuilder<
    N = field_state::NameNotAssigned,
    T = field_state::TypeNotAssigned,
> {
    name: Option<&'static str>,
    ty: Option<MetaType>,
    type_name: Option<&'static str>,
    docs: &'static [&'static str],
    marker: PhantomData<fn() -> (N, T)>,
}

impl<N, T> Default for FieldBuilder<N, T> {
    fn default() -> Self {
        FieldBuilder {
            name: Default::default(),
            ty: Default::default(),
            type_name: Default::default(),
            docs: Default::default(),
            marker: Default::default(),
        }
    }
}

impl FieldBuilder {
    /// Create a new FieldBuilder.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> FieldBuilder<field_state::NameNotAssigned, T> {
    /// Initialize the field name.
    pub fn name(self, name: &'static str) -> FieldBuilder<field_state::NameAssigned, T> {
        FieldBuilder {
            name: Some(name),
            ty: self.ty,
            type_name: self.type_name,
            docs: self.docs,
            marker: PhantomData,
        }
    }
}

impl<N> FieldBuilder<N, field_state::TypeNotAssigned> {
    /// Initialize the type of the field.
    pub fn ty<TY>(self) -> FieldBuilder<N, field_state::TypeAssigned>
    where
        TY: TypeInfo + 'static + ?Sized,
    {
        FieldBuilder {
            name: self.name,
            ty: Some(MetaType::new::<TY>()),
            type_name: self.type_name,
            docs: self.docs,
            marker: PhantomData,
        }
    }

    /// Initializes the type of the field as a compact type.
    pub fn compact<TY>(self) -> FieldBuilder<N, field_state::TypeAssigned>
    where
        TY: scale::HasCompact + TypeInfo + 'static,
    {
        FieldBuilder {
            name: self.name,
            ty: Some(MetaType::new::<scale::Compact<TY>>()),
            type_name: self.type_name,
            docs: self.docs,
            marker: PhantomData,
        }
    }
}

impl<N, T> FieldBuilder<N, T> {
    /// Initialize the type name of a field (optional).
    pub fn type_name(self, type_name: &'static str) -> FieldBuilder<N, T> {
        FieldBuilder {
            name: self.name,
            ty: self.ty,
            type_name: Some(type_name),
            docs: self.docs,
            marker: PhantomData,
        }
    }

    #[cfg(feature = "docs")]
    /// Initialize the documentation of a field (optional).
    pub fn docs(self, docs: &'static [&'static str]) -> FieldBuilder<N, T> {
        FieldBuilder {
            name: self.name,
            ty: self.ty,
            type_name: self.type_name,
            docs,
            marker: PhantomData,
        }
    }

    #[cfg(not(feature = "docs"))]
    #[inline]
    /// Doc capture is not enabled via the "docs" feature so this is a no-op.
    pub fn docs(self, _docs: &'static [&'static str]) -> FieldBuilder<N, T> {
        self
    }

    /// Initialize the documentation of a field, always captured even if the "docs" feature is not
    /// enabled.
    pub fn docs_always(self, docs: &'static [&'static str]) -> Self {
        FieldBuilder {
            name: self.name,
            ty: self.ty,
            type_name: self.type_name,
            docs,
            marker: PhantomData,
        }
    }
}

impl<N> FieldBuilder<N, field_state::TypeAssigned> {
    /// Complete building and return a new [`Field`].
    pub fn finalize(self) -> Field<MetaForm> {
        Field::new(
            self.name,
            self.ty.expect("Type should be set by builder"),
            self.type_name,
            self.docs,
        )
    }
}

/// Builds a definition of a variant type i.e an `enum`
#[derive(Default)]
#[must_use]
pub struct Variants {
    variants: Vec<Variant>,
}

impl Variants {
    /// Create a new [`VariantsBuilder`].
    pub fn new() -> Self {
        Variants {
            variants: Vec::new(),
        }
    }

    /// Add a variant
    pub fn variant<F>(mut self, name: &'static str, builder: F) -> Self
    where
        F: Fn(VariantBuilder) -> VariantBuilder<variant_state::IndexAssigned>,
    {
        let builder = builder(VariantBuilder::new(name));
        self.variants.push(builder.finalize());
        self
    }

    /// Add a unit variant (without fields).
    pub fn variant_unit(mut self, name: &'static str, index: u8) -> Self {
        let builder = VariantBuilder::new(name).index(index);
        self.variants.push(builder.finalize());
        self
    }

    /// Construct a new [`TypeDefVariant`] from the initialized builder variants.
    pub fn finalize(self) -> TypeDefVariant {
        TypeDefVariant::new(self.variants)
    }
}

/// State types for the `VariantBuilder` which requires an index.
pub mod variant_state {
    /// State where the builder has not assigned an index to a variant.
    pub enum IndexNotAssigned {}
    /// State where the builder has assigned an index to a variant.
    pub enum IndexAssigned {}
}

/// Build a [`Variant`].
#[must_use]
pub struct VariantBuilder<S = variant_state::IndexNotAssigned> {
    name: &'static str,
    index: Option<u8>,
    fields: Vec<Field<MetaForm>>,
    discriminant: Option<u64>,
    docs: Vec<&'static str>,
    marker: PhantomData<S>,
}

impl VariantBuilder<variant_state::IndexNotAssigned> {
    /// Create a new [`VariantBuilder`].
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            fields: Vec::new(),
            discriminant: None,
            index: None,
            docs: Vec::new(),
            marker: Default::default(),
        }
    }

    /// Set the variant's codec index.
    pub fn index(self, index: u8) -> VariantBuilder<variant_state::IndexAssigned> {
        VariantBuilder {
            name: self.name,
            index: Some(index),
            fields: self.fields,
            discriminant: self.discriminant,
            docs: self.docs,
            marker: Default::default(),
        }
    }
}

impl<S> VariantBuilder<S> {
    /// Set the variant's discriminant.
    pub fn discriminant(mut self, discriminant: u64) -> Self {
        self.discriminant = Some(discriminant);
        self
    }

    /// Initialize the variant's fields.
    pub fn fields<F>(mut self, fields_builder: FieldsBuilder<F>) -> Self {
        self.fields = fields_builder.finalize();
        self
    }

    #[cfg(feature = "docs")]
    /// Initialize the variant's documentation.
    pub fn docs(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }

    #[cfg(not(feature = "docs"))]
    #[inline]
    /// Doc capture is not enabled via the "docs" feature so this is a no-op.
    pub fn docs(self, _docs: &[&'static str]) -> Self {
        self
    }

    /// Initialize the variant's documentation, always captured even if the "docs" feature is not
    /// enabled.
    pub fn docs_always(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }
}

impl VariantBuilder<variant_state::IndexAssigned> {
    /// Complete building and create final [`Variant`] instance.
    pub fn finalize(self) -> Variant<MetaForm> {
        Variant::new(
            self.name,
            self.fields,
            self.index.expect("Index should be assigned by the builder"),
            self.docs,
        )
    }
}

use std::{sync::Arc, path::Path};

use bevy::{
    prelude::*,
    reflect::{GetTypeRegistration, ReflectRef, TypeRegistration},
    utils::HashMap, asset::{FileAssetIo, AssetIo},
};
use futures_lite::future;

use crate::{
    build_commands::BuildPrefabCommand, parse::parse_prefab_string, parse::LoadPrefabError,
    prefab::Prefab,
};

/// Manages and caches [Prefab] related data.
#[derive(Default, Resource)]
pub struct PrefabRegistry {
    type_data: HashMap<String, TypeInfo>,
    commands: HashMap<String, Arc<dyn BuildPrefabCommand + Send + Sync + 'static>>,
    prefabs: HashMap<String, Arc<Prefab>>,
}

impl PrefabRegistry {
    /// Register a component for use in a [Prefab].
    ///
    /// This must be called during setup on any component that gets loaded
    /// from a *.prefab* file. Prefab components must derive `Default` and `Reflect`
    /// and have the `#[reflect(Component)]` attribute.
    ///
    /// Note: Most built in bevy types are automatically registered during plugin
    /// initialization.
    ///
    /// ## Example
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_lazy_prefabs::*;
    ///
    /// #[derive(Default, Component, Reflect)]
    /// #[reflect(Component)]
    /// struct MyComponent {
    ///     i: i32,
    /// }
    ///
    /// fn setup(mut registry: ResMut<PrefabRegistry>) {
    ///     registry.register_type::<MyComponent>();
    /// }
    /// ```
    pub fn register_type<T: Reflect + GetTypeRegistration + Default>(&mut self) {
        let reg = T::get_type_registration();
        let instance = T::default();
        let name = reg.short_name().to_string();

        let info = TypeInfo {
            type_name: name.clone(),
            reflect_type: instance.reflect_ref().into(),
            registration: reg,
        };

        self.type_data.insert(name, info);
    }

    /// Register a [BuildPrefabCommand] for use in a [Prefab].
    ///
    /// This must be called during setup on any command that gets loaded
    /// from a *.prefab* file.
    pub fn register_build_command<T: BuildPrefabCommand + Default + Send + Sync + 'static>(
        &mut self,
    ) {
        let t = T::default();
        self.commands.insert(t.key().to_string(), Arc::new(t));
    }

    /// Load the [Prefab] from disk, or retrieve it if it's already been loaded.
    ///
    /// When first called for a prefab this will load it from disk and cache it internally.
    /// Future load calls for the same prefab will re-use this cached result.
    pub fn load(&mut self, name: &str) -> Result<&Arc<Prefab>, LoadPrefabError> {
        if self.prefabs.contains_key(name) {
            return Ok(self.prefabs.get(name).unwrap());
        };

        let io = FileAssetIo::new("assets/", false);
        let result = future::block_on(io.load_path(Path::new(name)));

        if let Err(error) = result {
            return Err(LoadPrefabError::FileReadError(error));
        }

        let data = result.unwrap();
        let prefab_string = String::from_utf8_lossy(&data);

        match parse_prefab_string(&prefab_string, self) {
            Ok(prefab) => {
                //let entry = self.prefab_map.entry(prefab_name.to_string());
                let entry = self.prefabs.entry(name.to_string());
                Ok(entry.or_insert_with(|| Arc::new(prefab)))
            }
            Err(e) => Err(e),
        }
    }

    /// Remove a cached [Prefab] from the registry.
    ///
    /// The next time the prefab is loaded it will be read from disk.
    pub fn unload_prefab(&mut self, name: &str) {
        self.prefabs.remove(name);
    }

    pub(crate) fn get_build_command(
        &self,
        name: &str,
    ) -> Option<&Arc<dyn BuildPrefabCommand + Send + Sync + 'static>> {
        self.commands.get(name)
    }

    pub(crate) fn get_type_data(&self, name: &str) -> Option<&TypeInfo> {
        self.type_data.get(name)
    }
}

pub(crate) struct TypeInfo {
    #[allow(dead_code)]
    pub type_name: String,
    pub reflect_type: ReflectType,
    pub registration: TypeRegistration,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum ReflectType {
    Struct,
    TupleStruct,
    Tuple,
    List,
    Map,
    Value,
    Array,
    Enum,
}

impl From<ReflectRef<'_>> for ReflectType {
    fn from(reflect: ReflectRef) -> Self {
        match reflect {
            ReflectRef::Struct(_) => ReflectType::Struct,
            ReflectRef::TupleStruct(_) => ReflectType::TupleStruct,
            ReflectRef::Tuple(_) => ReflectType::Tuple,
            ReflectRef::List(_) => ReflectType::List,
            ReflectRef::Map(_) => ReflectType::Map,
            ReflectRef::Value(_) => ReflectType::Value,
            ReflectRef::Array(_) => ReflectType::Array,
            ReflectRef::Enum(_) => ReflectType::Enum,
        }
    }
}

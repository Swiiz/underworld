use common::assets::CommonAssets;

pub struct ServerAssets {
    pub common: CommonAssets,
}

impl ServerAssets {
    pub fn load() -> Self {
        let common = CommonAssets::load();

        Self { common }
    }
}

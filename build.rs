use shadow_rs::ShadowBuilder;

fn main() {
    ShadowBuilder::builder()
        .build()
        .expect("Failed to build shadow");
}

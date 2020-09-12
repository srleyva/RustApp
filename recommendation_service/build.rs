fn main() -> Result<(),Box<dyn std::error::Error>> {
    tonic_build::configure()
        // It is included in the out/user.rs but the compiler says it can not find them.
        .type_attribute(".recommendation_svc.User", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".recommendation_svc.Location", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(".recommendation_svc.Location.longitude", "#[serde(rename = \"lon\")]")
        .field_attribute(".recommendation_svc.Location.latitude", "#[serde(rename = \"lat\")]")
        .compile(
            &["proto/recommendation/recommendation.proto"],
            &["proto/recommendation"]
        )?;
    Ok(())
}
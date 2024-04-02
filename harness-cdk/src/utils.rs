use ic_cdk::api::management_canister::{
    ecdsa::{ecdsa_public_key, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse},
    provisional::CanisterId,
};

pub async fn canister_public_key(
    canister_id: CanisterId,
    // TODO: create error types
) -> Result<EcdsaPublicKeyResponse, String> {
    let request = EcdsaPublicKeyArgument {
        canister_id: Some(canister_id),
        derivation_path: vec![],
        // todo: create proper key types
        key_id: Default::default(),
    };

    let (res,) = ecdsa_public_key(request)
        .await
        .map_err(|e| format!("ecdsa_public_key failed {:?}", e))?;

    Ok(res)
}

// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use futures::Future;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::EntryActions;
use rust_sodium::crypto::hash::sha256;
use rust_sodium::crypto::secretbox;
use safe_core::{Client, FutureExt, MDataInfo};
use safe_core::ipc::AppKeys;
use safe_core::utils::{symmetric_decrypt, symmetric_encrypt};
use super::{AccessContainerEntry, AuthError, AuthFuture};

/// Retrieves the authenticator configuration file
pub fn access_container(client: Client) -> Box<AuthFuture<MDataInfo>> {
    let parent = fry!(client.config_root_dir());
    let key = fry!(parent.enc_entry_key(b"access-container"));

    client.get_mdata_value(parent.name, parent.type_tag, key)
        .map_err(From::from)
        .and_then(move |val| {
            let content = parent.decrypt(&val.content)?;
            deserialise::<MDataInfo>(&content).map_err(From::from)
        })
        .into_box()
}

/// Encrypts and serialises an access container key using given app ID and app keys
pub fn access_container_key(access_container: &MDataInfo,
                            app_id: &str,
                            app_keys: &AppKeys)
                            -> Result<Vec<u8>, AuthError> {
    let nonce = if let Some((_, Some(nonce))) = access_container.enc_info {
        nonce
    } else {
        // No valid nonce for the MDataInfo could be found
        return Err(AuthError::Unexpected("No valid nonce for access container".to_owned()));
    };

    let key = app_id.as_bytes();

    let mut key_pt = key.to_vec();
    key_pt.extend_from_slice(&nonce[..]);

    let key_nonce =
        unwrap!(secretbox::Nonce::from_slice(&sha256::hash(&key_pt)[..secretbox::NONCEBYTES]));
    Ok(secretbox::seal(key, &key_nonce, &app_keys.enc_key))
}

/// Gets an access container entry
pub fn access_container_entry(client: Client,
                              access_container: &MDataInfo,
                              app_id: &str,
                              app_keys: AppKeys)
                              -> Box<AuthFuture<(u64, AccessContainerEntry)>> {
    let key = fry!(access_container_key(access_container, &app_id, &app_keys));

    client.get_mdata_value(access_container.name, access_container.type_tag, key)
        .and_then(move |value| {
            let plaintext = symmetric_decrypt(&value.content, &app_keys.enc_key)?;
            Ok((value.entry_version, deserialise(&plaintext)?))
        })
        .map_err(From::from)
        .into_box()
}

/// Adds a new entry to the authenticator access container
pub fn put_access_container_entry(client: Client,
                                  access_container: &MDataInfo,
                                  app_id: &str,
                                  app_keys: &AppKeys,
                                  permissions: AccessContainerEntry,
                                  version: Option<u64>)
                                  -> Box<AuthFuture<()>> {
    let key = fry!(access_container_key(access_container, app_id, app_keys));
    let plaintext = fry!(serialise(&permissions));
    let ciphertext = fry!(symmetric_encrypt(&plaintext, &app_keys.enc_key, None));

    let actions = if let Some(version) = version {
        EntryActions::new().update(key, ciphertext, version)
    } else {
        EntryActions::new().ins(key, ciphertext, 0)
    };

    client.mutate_mdata_entries(access_container.name,
                              access_container.type_tag,
                              actions.into())
        .map_err(From::from)
        .into_box()
}
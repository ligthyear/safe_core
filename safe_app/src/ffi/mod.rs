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

//! FFI

#![allow(unsafe_code)]

use ffi_utils::{FfiString, OpaqueCtx, catch_unwind_error_code};
use safe_core::ipc::AppExchangeInfo;
use safe_core::NetworkEvent;
use std::os::raw::c_void;
use super::App;
use super::errors::AppError;

/// Access container
pub mod access_container;
/// Cipher Options
pub mod cipher_opt;
/// Low level manipulation of `ImmutableData`
pub mod immutable_data;
/// IPC utilities
pub mod ipc;
/// `MDataInfo` operations
pub mod mdata_info;
/// Miscellaneous routines
pub mod misc;
/// Low level manipulation of `MutableData`
pub mod mutable_data;
/// NFS API
pub mod nfs;

mod helper;

/// Create unregistered app.
#[no_mangle]
pub unsafe extern "C" fn app_unregistered(user_data: *mut c_void,
                                          network_observer_cb: unsafe extern "C" fn(*mut c_void,
                                                                                    i32,
                                                                                    i32),
                                          o_app: *mut *mut App)
                                          -> i32 {
    catch_unwind_error_code(|| -> Result<_, AppError> {
        let user_data = OpaqueCtx(user_data);

        let app = App::unregistered(move |event| {
            call_network_observer(event, user_data.0, network_observer_cb)
        })?;

        *o_app = Box::into_raw(Box::new(app));

        Ok(())
    })
}

/// Create registered app from auth uri
#[no_mangle]
pub unsafe fn app_from_auth_uri(app_id: FfiString,
                                scope: FfiString,
                                name: FfiString,
                                vendor: FfiString,
                                uri: FfiString,
                                user_data: *mut c_void,
                                network_observer_cb: unsafe extern "C" fn(*mut c_void,
                                                                          i32,
                                                                          i32),
                                o_app: *mut *mut App)
                                -> i32 {
    catch_unwind_error_code(|| -> Result<_, AppError> {
        let user_data = OpaqueCtx(user_data);
        let uri = uri.to_string()?;
        let app_info = AppExchangeInfo {
            id: app_id.to_string()?,
            name: name.to_string()?,
            vendor: vendor.to_string()?,
            // an empty FfiString signals None for scope
            scope: if scope.len == 0 { None } else { Some(scope.to_string()?) }
        };
        let app = App::from_auth_uri(app_info, uri, move |event| {
            call_network_observer(event, user_data.0, network_observer_cb)
        })?;

        *o_app = Box::into_raw(Box::new(app));
        Ok(())
    })
}

unsafe fn call_network_observer(event: Result<NetworkEvent, AppError>,
                                user_data: *mut c_void,
                                cb: unsafe extern "C" fn(*mut c_void, i32, i32)) {
    match event {
        Ok(event) => cb(user_data, 0, event.into()),
        Err(err) => cb(user_data, ffi_error_code!(err), 0),
    }
}
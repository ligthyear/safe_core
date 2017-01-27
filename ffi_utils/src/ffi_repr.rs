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

//! FFI tools

/// Trait to convert between FFI and Rust representations of types
pub trait ReprC {
    /// C representation of the type
    type C;
    /// Error type
    type Error;

    /// Converts from a raw type into an owned type by cloning data
    fn from_repr_c_cloned(c_repr: Self::C) -> Result<Self, Self::Error> where Self: Sized;
}

impl ReprC for u64 {
    type C = u64;
    type Error = ();

    fn from_repr_c_cloned(c_repr: u64) -> Result<u64, ()> {
        Ok(c_repr)
    }
}

impl ReprC for usize {
    type C = usize;
    type Error = ();

    fn from_repr_c_cloned(c_repr: usize) -> Result<usize, ()> {
        Ok(c_repr)
    }
}

/// `XorName`
impl ReprC for [u8; 32] {
    type C = *const [u8; 32];
    type Error = ();

    #[allow(unsafe_code)]
    #[cfg_attr(feature="clippy", allow(not_unsafe_ptr_arg_deref))]
    fn from_repr_c_cloned(c_repr: *const [u8; 32]) -> Result<[u8; 32], ()> {
        unsafe { Ok(*c_repr) }
    }
}
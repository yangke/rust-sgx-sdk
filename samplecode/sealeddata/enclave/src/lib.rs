// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![crate_name = "enclave"]
#![crate_type = "staticlib"]

#![no_std]
#![feature(collections)]

#[macro_use]
extern crate collections;

extern crate sgx_types;
extern crate sgx_tseal;
extern crate sgx_trts;

use sgx_types::*;
use sgx_tseal::*;
use sgx_trts::*;

#[derive(Copy, Clone, Default, Debug)]
struct RandData {
    key: u32,
    rand: [u8; 16],
}

extern {
    fn ocall_print_string(str: * const c_uchar, len: size_t);
}

#[no_mangle]
pub extern "C" fn create_sealeddata(sealed_log: * mut u8, sealed_log_size: u32) -> sgx_status_t {

    let mut data = RandData::default();
    data.key = 0x1234;
    let error = rsgx_read_rand(&mut data.rand);
    if error.is_err() {
        return error.unwrap_err();
    } 

    let aad: [u8; 0] = [0_u8; 0];
    let result = SgxSealedData::<RandData>::seal_data(&aad, &data);
    let sealed_data = match result {
        Ok(x) => x,
        Err(ret) => {
            return ret;
        }, 
    };

    let opt = to_sealed_log(&sealed_data, sealed_log, sealed_log_size);
    if opt.is_none() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    
    let outstring = format!("{:?}", data);
    output(&outstring);
    
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn verify_sealeddata(sealed_log: * mut u8, sealed_log_size: u32) -> sgx_status_t {

    let opt = from_sealed_log::<RandData>(sealed_log, sealed_log_size);
    let sealed_data = match opt {
        Some(x) => x,
        None => {
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        },
    };
    
    let result = sealed_data.unseal_data();
    let unsealed_data = match result {
        Ok(x) => x,
        Err(ret) => {
            return ret;
        }, 
    };

    let data = unsealed_data.get_decrypt_txt();
    let outstring = format!("{:?}", data);
    output(&outstring);

    sgx_status_t::SGX_SUCCESS
}

fn output(outstr: &str) {
    unsafe {
        ocall_print_string(outstr.as_ptr() as *const c_uchar, outstr.len() as size_t);
    }
}

fn to_sealed_log<T>(sealed_data: &SgxSealedData<T>, sealed_log: * mut u8, sealed_log_size: u32) -> Option<* mut sgx_sealed_data_t> {
    unsafe {
        sealed_data.to_raw_sealed_data_t(sealed_log as * mut sgx_sealed_data_t, sealed_log_size)
    }
}
fn from_sealed_log<T: Copy>(sealed_log: * mut u8, sealed_log_size: u32) -> Option<SgxSealedData<T>> {
    unsafe {
        SgxSealedData::<T>::from_raw_sealed_data_t(sealed_log as * mut sgx_sealed_data_t, sealed_log_size)
    }
}
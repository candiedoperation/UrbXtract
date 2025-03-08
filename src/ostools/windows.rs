/*
    UrbXtract
    Copyright (C) 2025  Atheesh Thirumalairajan

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>. 
*/

use windows::core::PCSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{ReadFile, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeA, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT};
use std::ffi::CString;
use std::io;

pub(crate) struct WindowsSystemPipe {
    handle: HANDLE
}

impl WindowsSystemPipe {
    pub(crate) fn new(handle: HANDLE) -> Self {
        Self {
            handle
        }
    }

    pub(crate) fn await_clientconnect(&self) {
        unsafe {
            ConnectNamedPipe(
                self.handle, 
                None
            ).unwrap() 
        }
    }
}

impl io::Read for WindowsSystemPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        /* Define Bytes Read */
        let mut bytes_read = 0;
        
        unsafe {
            ReadFile(
                self.handle, 
                Some(buf), 
                Some(&mut bytes_read), 
                None
            ).unwrap();
        };

        /* Return Bytes Read */
        Ok(bytes_read as usize)
    }
}

pub(crate) fn create_named_pipe(pipe_name: &str, bufsize: u32) -> Result<WindowsSystemPipe, String> {
    unsafe {
        let pipename_cstr = CString::new(pipe_name).unwrap();
        let handle_res = CreateNamedPipeA(
            PCSTR(pipename_cstr.as_ptr() as *const u8),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,   // Number of instances
            0, // Out buffer size
            bufsize, // In buffer size
            0,   // Default timeout
            None,
        );

        match handle_res {
            Ok(handle) => Ok(WindowsSystemPipe::new(handle)),
            Err(e) => Err(format!("Failed to Create Named Pipe: {}", e.message())),
        }
    }
}
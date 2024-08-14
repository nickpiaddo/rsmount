// Copyright (c) 2023 Nick Piaddo
// SPDX-License-Identifier: Apache-2.0 OR MIT

// From dependency library

// From standard library

// From this library
use crate::gen_stepper;

gen_stepper!(Mount, ReMount, mnt_context_next_remount, MountInfoEntry);

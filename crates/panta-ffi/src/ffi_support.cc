#include "panta/ffi.hpp"
#include "rust/cxx.h"

namespace panta::ffi {

rust::String cpp_prefix() { return rust::String("ffi:"); }

} // namespace panta::ffi

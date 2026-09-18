#include "panta/ffi.hpp"

namespace panta::ffi {

rust::String cpp_prefix() { return rust::String("ffi:"); }

} // namespace panta::ffi

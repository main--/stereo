#include <mono/metadata/assembly.h>
#include <mono/metadata/mono-gc.h>
#include <mono/metadata/threads.h>
#include <mono/metadata/debug-helpers.h>
#include <mono/metadata/tokentype.h>
#include <mono/jit/jit.h>


// Don't want to pull in all of glibc just for this small helper
void g_free (void* mem);

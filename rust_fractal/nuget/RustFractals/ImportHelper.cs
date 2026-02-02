using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

namespace RustFractals
{
    public static unsafe partial class NativeMethods
    {
        static NativeMethods()
        {
            NativeLibrary.SetDllImportResolver(
                typeof(NativeMethods).Assembly,
                DllImportResolver
            );
        }

        private static IntPtr DllImportResolver(
            string libraryName,
            Assembly assembly,
            DllImportSearchPath? searchPath)
        {
            if (libraryName != __DllName)
                return IntPtr.Zero;

            // 1️⃣ Preferred path: let .NET / OS resolve packaged native assets
            if (NativeLibrary.TryLoad(libraryName, assembly, searchPath, out var handle))
                return handle;

            // 2️⃣ DEV-ONLY fallback (Windows): probe known relative layout
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                var rid = RuntimeInformation.RuntimeIdentifier; // e.g. "win-x64"
                var fileName = $"{libraryName}.dll";

                var candidate = Path.Combine(
                    AppContext.BaseDirectory,
                    "runtimes",
                    rid,
                    "native",
                    fileName
                );

                if (File.Exists(candidate))
                    return NativeLibrary.Load(candidate);
            }

            throw new DllNotFoundException(
                $"Unable to load native library '{libraryName}'. " +
                $"Default probing failed."
            );
        }
    }
}


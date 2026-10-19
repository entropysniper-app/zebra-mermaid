# Third-party notices

zebra-mermaid is distributed under the MIT license. Parts of it are derived from the third-party
sources listed below, under the terms quoted for each.

## Pele

The renderer (`crates/zebra-mermaid/src/**`) is a Rust port of Pele 1.6.0
(<https://github.com/obsidianmd/pele>, commit `44da152bbb79b91f03e68016f42499bb25bf6e82`):
its preprocessing, YAML reader, parsers, layout, text measurement tables and SVG output. Pele's
Arial advance table (`text/metrics.rs`) and Unicode letter table (`util/unicode.rs`) are copied
from that commit. The differential fixtures under `tests/fixtures/` hold inputs from Pele's test
suite and corpus with the output Pele produces for them. `tests/upstream/source/tests/` is a
verbatim copy of the pinned test suite and its support files; `crates/zebra-mermaid/tests/upstream/`
contains direct Rust ports, mapped individually in `tests/upstream/ports.json`. Original copyright
headers, including Mermaid-derived tests, are preserved.

```
MIT License

Copyright (c) 2026 kepano

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

Pele's browser and zoom implementation has not yet been ported. Its tests are retained in the
full inventory and must not be counted as implemented. The zoom implementation's Lucide icon
license must be included if that asset is ported.

## Mermaid

Pele's grammars, parser behavior and test corpus follow Mermaid 12.1.0
(<https://github.com/mermaid-js/mermaid>, commit prefix `21f72f07ea`); the corpus fixtures in
`tests/fixtures/corpus/` contain Mermaid documentation examples.

```
MIT License

Copyright (c) 2014 - 2022 Knut Sveidqvist

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Zebra theme contract

`crates/zebra-mermaid/src/theme/` and `src/zebra.rs`, and `crates/zebra-mermaid-ffi`, follow the
palette contract, default palettes, color mixing, label contrast rule and C ABI (version 2) of
Zebra's native Mermaid bridge (`Vendor/ZebraMermaidBridge`, Zebra `c21cba1b`), which this project
is built to stay compatible with; `crates/zebra-mermaid-ffi/include/zebra_mermaid.h` is a copy of
its `ZebraMermaidBridge.h`. Check that Zebra code's license terms before distributing these parts
outside the Zebra project.

## `Math` functions: fdlibm, V8 and musl

`crates/zebra-mermaid/src/util/fdlibm.rs` is a Rust port of the fdlibm functions in V8's
`src/base/ieee754.cc` (V8 14.6.202.34, as bundled in Node.js v26.3.0:
<https://github.com/nodejs/node/blob/v26.3.0/deps/v8/src/base/ieee754.cc>): `sin`, `cos`, `tan`,
`asin`, `acos`, `atan`, `atan2`, `exp`, `log`, `log2` and `log10` with their kernels
(`__ieee754_rem_pio2`, `__kernel_rem_pio2`, `__kernel_sin`, `__kernel_cos`, `__kernel_tan`,
`k_log1p`). `log2` and `k_log1p` come from FreeBSD msun's `e_log2.c` and `k_log.h` as copied into
that V8 file. Its `scalbn` helper follows musl's `src/math/scalbn.c`.

### fdlibm

```
Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.

Developed at SunSoft, a Sun Microsystems, Inc. business.
Permission to use, copy, modify, and distribute this
software is freely granted, provided that this notice
is preserved.
```

### V8

V8's file states: "The original source code covered by the above license above has been modified
significantly by Google Inc. Copyright 2016 the V8 project authors. All rights reserved." V8 is
distributed under this license (<https://github.com/nodejs/node/blob/v26.3.0/deps/v8/LICENSE>):

```
Copyright 2014, the V8 project authors. All rights reserved.
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above
      copyright notice, this list of conditions and the following
      disclaimer in the documentation and/or other materials provided
      with the distribution.
    * Neither the name of Google Inc. nor the names of its
      contributors may be used to endorse or promote products derived
      from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

### musl (`scalbn`)

From <https://git.musl-libc.org/cgit/musl/tree/COPYRIGHT>:

```
Copyright © 2005-2020 Rich Felker, et al.

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

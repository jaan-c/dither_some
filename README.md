# dither_some
Dither images and videos using various algorithms.

```
Usage: dither_some --algorithm <ALGORITHM> --palette-count <PALETTE_COUNT> <INPUT> <OUTPUT>

Arguments:
<INPUT>   Path of video to dither
<OUTPUT>  Path where to store dithered video

Options:                                                                                                                  
-a, --algorithm <ALGORITHM>          Algorithm to be used for dithering [possible values: atkinson, fs-color]           
-p, --palette-count <PALETTE_COUNT>  Restricts palette by specified count                                               
-h, --help                           Print help                                                                         
-V, --version                        Print version
```
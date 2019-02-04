# Benchmarks for image crate pixel iterator
This benchmark computes the sum of all rgb components in the image.
It measures the time and memory usage.
It does this using `ImageBuffer`, a streaming iterator optimized for RGB8 and
a streaming iterator with generic pixel type.

Run with: 
```
> cargo run --release path/to/image.png
```

For [this nasa image](https://eoimages.gsfc.nasa.gov/images/imagerecords/73000/73751/world.topo.bathy.200407.3x21600x10800.png)
of 21600x10800 rgb pixels the results on my machine are:

```
Buffer:
pixel sum: [3341899515, 1612940354, 675870307]
time used: 7.231880666s, memory allocated (in kb): 683855

Incremental static:
pixel sum: [3341899515, 1612940354, 675870307]
time used: 8.606524099s, memory allocated (in kb): 490

Incremental dynamic:
pixel sum: [3341899515, 1612940354, 675870307]
time used: 9.765532836s, memory allocated (in kb): 490
```

Unsurprisingly the `ImageBuffer` version is slightly faster, but
uses a lot more memory, while the 

# rapax
The #1 best semi-safe OpenGL abstraction crate. Read more to see why.

## Problems with other abstraction crates (`miniquad`, `glium`, `notan`, etc.)

1. Most existing OpenGL abstraction crates lack important features. For example, `miniquad` and `notan` do not allow you to create a depth stencil buffer.
2. Many OpenGL abstraction crates lack web support (mainly `glium`).
3. Many OpenGL abstraction crates are unmaintained (`glium`, and one other one which I cannot remember the name of).
5. Many OpenGL abstraction crates do not provide ways to import foreign textures. `notan` does have a way to add textures to the context, but this mechanism, doesn't allow for the use of extensions such as `GLX_EXT_texture_from_pixmap`.

## Why not `wgpu`?

`wgpu` has many of the same problems as the crates listed above. Namely:

1. wgpu provides no way to import foreign textures.
2. wgpu *does* have web support, however it is poor and buggy in my experience.

Furthermore, wgpu is poorly supported on older machines with older GPUs, making it a poor fit for games intended for low-end hardware.

## Why not `rafx`?

I have not looked into rafx too much, though two issues stick out:
1. No foreign texture API
2. As a Vulkan-esque API, it is poorly supported on older machines

## How Rapax addresses these issues

Rapax addresses these problems by making it very easy to access internal OpenGL state without meddling with too many internals. If Rapax doesn't have a feature you need (I guarantee this will almost never happen) it's easy to hack it in without touching any Rapax code.
Using a foreign texture is as easy as:
```rs
ctx.bind_texture(0, foreign_texture_name);```
Similarly, using a foreign shader is as easy as:
```rs
ctx.use_program(shader_name);```
I'm sure you get the point by now. All of Rapax's context functions take in Rapax's managed primitives in addition to raw names/handles.

In the case of extensions such as `GLX_EXT_texture_from_pixmap` it's as easy as:
```rs
ctx.flush_state(); // more on this function later
gl.active_texture(glow::TEXTURE0);
glXBindTexImageEXT(...);
ctx.invalidate_texture(0); // the input is the currently selected texture unit, this must be done when you make a foreign binding call
ctx.draw_elements(...);
glXReleaseTexImageEXT(...);
```

## Caveats
One caveat to keep in mind when touching internal OpenGL is that Rapax caches internal state for performance reasons. Before drawing anything, it is on the user to call `ctx.flush_state()` to flush cached state to the OpenGL context. Manual flushing is a consequence of Rapax's hackability.

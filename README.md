# rapax
The #1 best safe & stateless OpenGL abstraction crate, providing RAII wrappers and a simpler Rusty interface. Read more to see why.

## Features

* RAII handles to OpenGL objects (buffers, textures, framebuffers).
* Abstracted pipeline API.
* Depth & stencil operations.
* High-level vertex attribute API.
* Easy to incorporate foreign objects.

## Problems with other abstraction crates (miniquad, glium, notan, etc.)

1. Most existing OpenGL abstraction crates lack important features. For example, miniquad and notan do not allow you to create a depth stencil buffer.
2. Many OpenGL abstraction crates lack web support (mainly glium).
3. Many OpenGL abstraction crates are unmaintained.
5. Many OpenGL abstraction crates do not provide ways to import foreign textures. notan does have a way to add textures to the context, but this mechanism doesn't allow for the use of extensions such as `GLX_EXT_texture_from_pixmap`.

## Why not wgpu?

wgpu has many of the same problems as the crates listed above. Namely:

1. wgpu provides no way to import foreign textures.
2. wgpu *does* have web support via WebGL, however it is poor and buggy in my experience.

Furthermore, wgpu is poorly supported on older machines with older GPUs, making it a poor fit for games intended for low-end hardware.

## Why not rafx?

I have not looked into rafx too much, though two issues stick out:
1. No foreign texture API
2. As a Vulkan-esque API, it is poorly supported on older machines

## How Rapax addresses these issues

Rapax addresses these problems by making it very easy to access internal OpenGL state without meddling with too many internals. If Rapax doesn't have a feature you need (I guarantee this will almost never happen) it's easy to hack it in without touching any Rapax code.
Using a foreign texture is as easy as:
```rs
ctx.apply_textures(&[
    (foreign_texture_name, "uTexture"),
]);
```
All of Rapax's context functions take in Rapax's managed primitives in addition to raw OpenGL names/handles. This is controlled using the `Bindable*` series of traits.

In the case of extensions such as `GLX_EXT_texture_from_pixmap` it's as easy as:
```rs
struct Pixmap(...);

impl rapax::BindableTexture for Pixmap {
    unsafe fn bind(&self) {
        glXBindTexImageEXT(...);
    }
}

impl Pixmap {
    unsafe fn release(&self) {
        glXReleaseTexImageEXT(...);
    }
}

ctx.with_pipeline(&some_pipeline, |rctx| {
    let foreign_texture_name = Pixmap(some_x11_thing);
    rctx.apply_textures(&[
        (foreign_texture_name, "uTexture"),
    ]);
    rctx.draw_elements(rapax::DrawMode::Triangles, 100, DataType::UnsignedShort, 0);
    unsafe { foreign_texture_name.release() };
})
```
Rapax does not cache any internal GL state, so there is no way to put the context into an unusable state by binding textures/programs/buffers manually.
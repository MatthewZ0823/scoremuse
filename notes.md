Notes if I ever decide to write a blog or something

# Motivation

I want to create a piece of music notation software that works smoother than
Musescore, especially with larger files. I also want to add some features to
make transcription/arranging easier, such as being able to import audio files
and overlay them on the score, with a spectrogram or something. I also want to
make a p2p system where people can collab on a score in real time

# Technology

I wanted to write this in Rust, for the preformance, but mostly for the memes.
Rust doesn't have that many UI frameworks (who would've thought). I chose Iced
because of its Elm like architecture and canvas sub-module. Having a UI
framework will be nice for the buttons and stuff that isn't the score itself,
but for the score I will be more on my own, just using the experimental canvas
sub-module.

# Progress

I've never used Iced before, or Rust to build a project, so I decided a good
place to start was to try and render some simple score looking thing. I was
able to render a staff and some svgs for the clef and notes. I found that the
tools in the canvas module weren't as good as I'd like them to be. There was no
obvious way of compositing transformations/shapes, or determining re-rendering.
I was going to build my own API to fix this, but as I thought about it more, I
realize I would basically have to create React for something that doesn't need
to be that general, so I got discouraged from the project.

Next, I tried to approach the project from a data-structures first point of
view. This is more in line with the Elm style of thinking. Doing some research
into what data-structures I was going to need. I figured that the
data-structures I would use here would be similar to those of text editors, so
I started researching those

https://cdacamar.github.io/data%20structures/algorithms/benchmarking/text%20editors/c++/editor-data-structures/

Just kidding, I'm not going to do any advanced datastructures, just the simplest ones I can think of that can model what I need. Right now I can render a simple bar, and have a better feel of the canvas module. Let's work on rendering multiple bars

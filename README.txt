
        Play Me Some ... Picross!


About
-----

    A simple Picross game written in Rust.

    Features:
    - undo and redo
    - playable at almost all resolutions
    - sexy graphics!


Compiling
---------

    Requires SDL2 from your distribution.
    Other dependencies will be handled by cargo.

    To build, run:

        cargo build


Starting up
-----------

    To play, run:

        cargo run <optional puzzle file>

    Alternatively, run the executable found inside the 'target' directory.

    The data file, picross.png, should be in either:
    1. current_directory/resources/picross.png, or
    2. the same directory as the executable.

    An example puzzle file:

        # width x height
        8 x 10

        # rows
        6
        2 2
        1 1 1 1
        1 1
        1 1
        1 4 1
        1 2 1
        1 1
        2 2
        6

        # columns
        8
        2 2
        1 1 1 1
        1 2 1
        1 2 1
        1 1 1 1
        2 2
        8

    You can also drag-and-drop a puzzle file to load it.


Controls
--------

    z - undo
    x - redo
    a - auto-fill
    1-3 - select paint

    lmb - paint tile, or cross out
    rmb - clear tile
    wheel - zoom
    mouse thumb buttons - undo, redo

    F11, f - toggle fullscreen


Author
------

David Wang <millimillenary@gmail.com>

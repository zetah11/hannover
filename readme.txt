HANNOVER     IS FOR CREATING LIVE MUSIC
---------------------------------------

HANNOVER is an environment for the live
creation of procedural music. As a user
you input text that the program
interprets as commands, pitches,
durations, and so on.

This is achieved by encoding the text
with UTF-8 into a series of bytes. The
bytes are then forcibly separated into
streams of nibbles (one nibble being
half a byte). These nibble streams are
taken apart, inspected, and used by
various modules.

HANNOVER is at its core a monomorphic
wavetable synthesizer.

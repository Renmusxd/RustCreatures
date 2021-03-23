# RustCreatures

Simulate a little evolution at home.
Requires sdl2, sdl2-gfx, and OpenBLAS.

How it works:
Each creature has a randomly assigned neural network that takes visual inputs (5 angles with distance+color, press V to see line of sight), neighboring grass information (9 tiles total) and own total energy then maps these to movement and action choices.
The creatures can choose to move forward and/or turn, as well as eat, replicate, bite, or nothing. Energy costs increase with movement and the eating of grass or biting other creatures adds to their energy. Replication costs a fix amount of energy to produces a clone with slight mutations to the neural network making action choices.

The world will keep a minimum total population as well as a minimum number of distinct families.

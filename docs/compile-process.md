# Compile Process

The different stages that make up the pipeline for compiling a map are:

1. **Map parsing:** converting a source file on disk, in a particular format, into a collection of entities, brushes and faces. This transforms a **map input file** into a **map source file**.
2. **CSG (Constructive Solid Geometry):** converting the clusters of map faces into convex volumes and renderable polygons. This transforms a **map source file** into a **map CSG file**.
3. **BSP (Binary Space Partitioning):** splitting the static map geometry into a binary tree based on the map's face planes. This transforms a **map CSG file** into a **map BSP file**.
4. **VIS (Visibility Computation):** computing which areas in a map are visible from which other areas. This adds **visibility data** to an existing **map BSP file**.
5. **RAD (Radiosity Simulation):** simulating lighting effects in the map, and baking light levels into the structure of the map. This adds **lighting data** to an existing **map BSP file**.
6. **Map serialisation:** taking all computed data and writing it to a BSP file on disk, in a particular format that a game expects. This converts a **map BSP file** into a **map output file**.

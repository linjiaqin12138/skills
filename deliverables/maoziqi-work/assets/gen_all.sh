#!/bin/bash
cd "$(dirname "$0")"
run() {
  for i in 1 2 3 4 5 6; do
    python3 gen.py "$2" "$1" "$3" && break
    sleep 50
  done
  sleep 44
}
BASE="photorealistic AI-generated video still, cinematic, autumn in southern China, Guangdong mountains, layered misty green ridges, warm late-afternoon sunlight, shallow depth of field, ultra detailed fur, vertical 9:16, no text, no watermark, no logo, no signage"
run shot1.jpg "$BASE, two cats seen from behind hiking up a mountain stone path, a plump orange tabby cat and a fluffy white cat with heterochromia one blue eye one amber eye, each wearing a tiny cloth backpack, autumn leaves on the ground, bamboo and pine forest, early morning haze" 201
run shot2.jpg "$BASE, a plump orange tabby cat walking up mossy stone steps through a bamboo forest, warm light beams through bamboo leaves, fallen red maple leaves, small hiking backpack on its back, low angle, cinematic" 202
run shot3.jpg "$BASE, a fluffy white cat with heterochromia one blue eye one amber eye sitting on a rock at a mountain viewpoint, looking at endless layered mountain ridges and a sea of clouds, back view, warm rim light" 203
run shot4.jpg "$BASE, a fluffy white cat with heterochromia one blue eye one amber eye drinking from a clear mountain stream over mossy rocks, autumn maple leaves floating, dappled sunlight, macro detail" 204
run shot5.jpg "$BASE, a chubby gray cat wearing a mustard yellow knit beanie and a tiny backpack standing on a windy grassy mountain summit, looking at the horizon, wind blown fur and grass, dramatic clouds" 205
run shot6.jpg "$BASE, three cats sitting side by side on a mountain summit meadow at golden hour, a plump orange tabby, a fluffy white cat with heterochromia one blue eye one amber eye, a chubby gray cat wearing a mustard yellow knit beanie, warm sunset glow, wide landscape behind" 206
run shot7.jpg "$BASE, three cats descending a winding mountain trail at sunset, long warm shadows on the path, back view, an orange tabby, a white cat with heterochromia eyes, a gray cat in a mustard beanie, golden hour backlight, cinematic" 207

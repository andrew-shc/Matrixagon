/*
Using the Perlin noise for the majority of the terrain and the heightmap
 */


use oorandom::Rand64;


#[derive(Clone)]
pub struct PerlinNoise2D {
    random: Rand64,  // random instance
    // permutation
    p: Vec<u32>,  // permutation to additionally randomize
    // gradient vectors
    g: Vec<(f64, f64)>,  // selected gradient vectors to be used in the noise
}

impl PerlinNoise2D {
    // this struct is to be only instantiated each time a new terrain is instantiated

    pub fn new(seed: u128) -> Self {
        let rand = Rand64::new(seed);
        Self {
            random: rand,
            p: Self::permutation(rand, 256),
            // the more different gradient, the more variation the noise has
            g: vec![
                ( 1.0, 1.0),
                ( 1.0,-1.0),
                (-1.0, 1.0),
                (-1.0,-1.0),

                ( 1.0, 0.0),
                (-1.0, 0.0),
                ( 0.0, 1.0),
                ( 0.0,-1.0),
            ],
        }
    }

    pub fn perlin(&self, x: f64, y: f64) -> f64 {
        let (xf1, yf1) = (x.ceil(), y.ceil());
        let (xf0, yf0) = (x.floor(), y.floor());

        let c00 = (xf0, yf0);
        let c01 = (xf0, yf1);
        let c10 = (xf1, yf0);
        let c11 = (xf1, yf1);

        // retrieves the gradient base off positon
        let grad = |x, y| {
            self.g[self.p[(self.p[(x as u32 & 255) as usize]+(y as u32 & 255) & 255) as usize] as usize % self.g.len()]
        };

        let g00 = grad(xf0, yf0);
        let g01 = grad(xf0, yf1);
        let g10 = grad(xf1, yf0);
        let g11 = grad(xf1, yf1);

        // dot product between (xf0,yf0) and gradient vectors
        let d00 = (c00.0*g00.0) + (c00.1*g00.1);
        let d01 = (c01.0*g01.0) + (c01.1*g01.1);
        let d10 = (c10.0*g10.0) + (c10.1*g10.1);
        let d11 = (c11.0*g11.0) + (c00.1*g11.1);

        let (u, v) = (Self::fade(x-xf0 as f64), Self::fade(y-yf0 as f64));

        // the ordering of the lerp is very important
        Self::lerp(
            v,
            Self::lerp(u, d00, d10),
            Self::lerp(u, d01, d11),
        )
    }

    // Fisher-Yates shuffle: Durstenfield's version
    fn permutation(mut random: Rand64, n: u32) -> Vec<u32> {
        let mut result = Vec::with_capacity(n as usize);
        let mut scratch = (1..n+1).collect::<Vec<_>>();

        while scratch.len() > 1 {
            let roll = random.rand_range(0..scratch.len() as u64) as usize;

            // moves the chosen number to append it to result
            result.push(scratch[roll]);
            // moves the last number to the chosen number
            scratch.swap_remove(roll);

        }
        // once most of the value has been moved, we assume the scratch only has one element left
        // thus we push the last element to the end of our result
        result.push(scratch[0]);

        result
    }

    // linear interpolation
    fn lerp(t: f64, v1: f64, v2: f64) -> f64 {
        v1 + t*(v2-v1)
    }

    fn fade(t: f64) -> f64 {
        6.0*t*t*t*t*t - 15.0*t*t*t*t + 10.0*t*t*t
    }
}

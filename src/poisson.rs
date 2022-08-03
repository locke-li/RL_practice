

pub struct Poisson {
    pmf_v: Vec<f64>,
    cdf_v: Vec<f64>,
    pub l:usize,
}

impl Poisson {
    pub fn new(l:usize, range:usize) -> Self {
        let mut pmf_v:Vec<f64> = Vec::new();
        let mut cdf_v:Vec<f64> = Vec::new();
        let e:f64 = 2.7182818284;
        let lf = l as f64;
        let mut n_rank:f64 = 1.0;
        let mut cdf:f64 = 0.0;
        for n in 0..=range {
            let nf = n as f64;
            if n > 0 { n_rank *= nf }
            let p = e.powf(-lf) * lf.powf(nf) / n_rank;
            pmf_v.push(p);
            cdf += p;
            cdf_v.push(cdf);
        }
        Self { l, pmf_v, cdf_v }
    }

    pub fn pmf(&self, v:usize) -> f64 {
        self.pmf_v[v]
    }

    pub fn cdf(&self, v:usize) -> f64 {
        self.cdf_v[v]
    }
}
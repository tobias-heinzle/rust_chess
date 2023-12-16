use dfdx::prelude::*;

type Network = (Linear<3, 5>, ReLU, Linear<5, 10>);

pub fn tensor_test(){
    let dev: Cpu = Default::default();
    let t: Tensor<Rank2<2, 3>, f32, _> = dev.zeros();

    let model = dev.build_module::<Network, f32>();

    let x: Tensor<(usize, Const<3>), f32, _> = dev.sample_normal_like(&(10, Const));
    let y = model.forward(x);

    println!("{:?}", y);
}
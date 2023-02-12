/*
 * Copyright 2023 Asim Ihsan
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use tch::Tensor;

pub fn grad_example() {
    let mut x = Tensor::from(2.0f32)
        .to_device(tch::Device::Mps)
        .set_requires_grad(true);
    let y = &x * &x + &x + 36;
    println!("{}", y.double_value(&[]));
    x.zero_grad();
    y.backward();
    let dy_over_dx = x.grad();
    println!("{}", dy_over_dx.double_value(&[]));
}

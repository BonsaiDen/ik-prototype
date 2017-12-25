// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Modules --------------------------------------------------------------------
mod standard_rifle;
pub use self::standard_rifle::StandardRifle;

mod stick_figure;
pub use self::stick_figure::{
    StickFigureConfig, StickFigureState, StickFigure, StickFigureRenderer
};


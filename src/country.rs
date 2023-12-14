// Copyright (C) 2017 1aim GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Country related types.

use serde_derive::{Deserialize, Serialize};
use std::str;
use strum::{AsRefStr, EnumString};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Debug)]
pub struct Code {
    /// The country code value.
    pub(crate) value: u16,

    /// The source from which the country code is derived.
    pub(crate) source: Source,
}

/// The source from which the country code is derived. This is not set in the
/// general parsing method, but in the method that parses and keeps raw_input.
#[derive(Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The country code is derived based on a phone number with a leading "+",
    /// e.g. the French number "+33 1 42 68 53 00".
    Plus,

    /// The country code is derived based on a phone number with a leading IDD,
    /// e.g. the French number "011 33 1 42 68 53 00", as it is dialled from US.
    Idd,

    /// The country code is derived based on a phone number without a leading
    /// "+", e.g. the French number "33 1 42 68 53 00" when the default country
    /// is supplied as France.
    Number,

    /// The country code is derived NOT based on the phone number itself, but
    /// from the default country parameter provided in the parsing function by
    /// the clients. This happens mostly for numbers written in the national
    /// format (without country code). For example, this would be set when
    /// parsing the French number "01 42 68 53 00", when the default country is
    /// supplied as France.
    Default,
}

impl Default for Source {
    fn default() -> Self {
        Source::Default
    }
}

impl Code {
    /// The country code number.
    pub fn value(&self) -> u16 {
        self.value
    }

    /// How the country was inferred.
    pub fn source(&self) -> Source {
        self.source
    }
}

impl From<Code> for u16 {
    fn from(code: Code) -> u16 {
        code.value
    }
}

/// CLDR country IDs.
#[derive(Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Hash, Debug, EnumString, AsRefStr)]
pub enum Id {
    AC,
    AD,
    AE,
    AF,
    AG,
    AI,
    AL,
    AM,
    AO,
    AR,
    AS,
    AT,
    AU,
    AW,
    AX,
    AZ,
    BA,
    BB,
    BD,
    BE,
    BF,
    BG,
    BH,
    BI,
    BJ,
    BL,
    BM,
    BN,
    BO,
    BQ,
    BR,
    BS,
    BT,
    BW,
    BY,
    BZ,
    CA,
    CC,
    CD,
    CF,
    CG,
    CH,
    CI,
    CK,
    CL,
    CM,
    CN,
    CO,
    CR,
    CU,
    CV,
    CW,
    CX,
    CY,
    CZ,
    DE,
    DJ,
    DK,
    DM,
    DO,
    DZ,
    EC,
    EE,
    EG,
    EH,
    ER,
    ES,
    ET,
    FI,
    FJ,
    FK,
    FM,
    FO,
    FR,
    GA,
    GB,
    GD,
    GE,
    GF,
    GG,
    GH,
    GI,
    GL,
    GM,
    GN,
    GP,
    GQ,
    GR,
    GT,
    GU,
    GW,
    GY,
    HK,
    HN,
    HR,
    HT,
    HU,
    ID,
    IE,
    IL,
    IM,
    IN,
    IO,
    IQ,
    IR,
    IS,
    IT,
    JE,
    JM,
    JO,
    JP,
    KE,
    KG,
    KH,
    KI,
    KM,
    KN,
    KP,
    KR,
    KW,
    KY,
    KZ,
    LA,
    LB,
    LC,
    LI,
    LK,
    LR,
    LS,
    LT,
    LU,
    LV,
    LY,
    MA,
    MC,
    MD,
    ME,
    MF,
    MG,
    MH,
    MK,
    ML,
    MM,
    MN,
    MO,
    MP,
    MQ,
    MR,
    MS,
    MT,
    MU,
    MV,
    MW,
    MX,
    MY,
    MZ,
    NA,
    NC,
    NE,
    NF,
    NG,
    NI,
    NL,
    NO,
    NP,
    NR,
    NU,
    NZ,
    OM,
    PA,
    PE,
    PF,
    PG,
    PH,
    PK,
    PL,
    PM,
    PR,
    PS,
    PT,
    PW,
    PY,
    QA,
    RE,
    RO,
    RS,
    RU,
    RW,
    SA,
    SB,
    SC,
    SD,
    SE,
    SG,
    SH,
    SI,
    SJ,
    SK,
    SL,
    SM,
    SN,
    SO,
    SR,
    SS,
    ST,
    SV,
    SX,
    SY,
    SZ,
    TA,
    TC,
    TD,
    TG,
    TH,
    TJ,
    TK,
    TL,
    TM,
    TN,
    TO,
    TR,
    TT,
    TV,
    TW,
    TZ,
    UA,
    UG,
    US,
    UY,
    UZ,
    VA,
    VC,
    VE,
    VG,
    VI,
    VN,
    VU,
    WF,
    WS,
    XK,
    YE,
    YT,
    ZA,
    ZM,
    ZW,
}

pub use Id::*;

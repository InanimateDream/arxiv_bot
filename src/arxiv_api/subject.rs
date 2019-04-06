use super::*;
use Subject::*;

// TODO: complete the type
#[derive(Clone, Debug, PartialEq)]
pub enum Subject {
    math_CT,
    math_HO,
    math_LO,
    cs_LO,
    cs_PL,
    Others(String),
}

const MATH_CT: &str = "math.CT";
const MATH_HO: &str = "math.HO";
const MATH_LO: &str = "math.LO";
const CS_LO: &str = "cs.LO";
const CS_PL: &str = "cs.PL";

const MATH_CT_DP: &str = "Category Theory (math.CT)";
const MATH_HO_DP: &str = "History and Overview (math.HO)";
const MATH_LO_DP: &str = "Logic (math.LO)";
const CS_LO_DP: &str = "Logic in Computer Science (cs.LO)";
const CS_PL_DP: &str = "Programming Languages (cs.PL)";

impl FromStr for Subject {
    type Err = Error;
    fn from_str(sub: &str) -> Fallible<Self> {
        match sub {
            MATH_CT | MATH_CT_DP => Ok(math_CT),
            MATH_HO | MATH_HO_DP => Ok(math_HO),
            MATH_LO | MATH_LO_DP => Ok(math_LO),
            CS_LO | CS_LO_DP => Ok(cs_LO),
            CS_PL | CS_PL_DP => Ok(cs_PL),
            _ => Ok(Others(sub.to_owned()))
        }
    }
}

impl Subject {
    pub fn as_str(&self) -> &str {
        match self {
            math_CT => MATH_CT,
            math_HO => MATH_HO,
            math_LO => MATH_LO,
            cs_LO => CS_LO,
            cs_PL => CS_PL,
            Others(s) => Subject::strip(s),
        }
    }

    fn strip(s: &str) -> &str {
        s.split(|c| c == '(' || c == ')').collect::<Vec<_>>()[1]
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            math_CT => MATH_CT_DP,
            math_HO => MATH_HO_DP,
            math_LO => MATH_LO_DP,
            cs_LO => CS_LO_DP,
            cs_PL => CS_PL_DP,
            Others(s) => s,
        };
        write!(f, "{}", s)
    }
}
/** Struct comment */
type itemDetailsFieldValue {}

/** Enum comment */
type advancedColors = 
    /** This is a case comment */
    | String(string),
    | Number(integer),
    | UnsignedNumber(float),
    | NumberArray(Vec<integer>),
    /** Comment on the last element */
    | ReallyCoolType(itemDetailsFieldValue),
}

type advancedColors2 =
	  /** This is a case comment */
    | String(string),
    | Number(float),
    | NumberArray(Vec<float>),
	  /** Comment on the last element */
    | ReallyCoolType(itemDetailsFieldValue),
}

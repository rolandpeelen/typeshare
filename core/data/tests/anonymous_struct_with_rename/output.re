type AnonymousStructWithRename = 
  | List(array(string))
  | LongFieldNames({
      someLongFieldName: string,
      and: bool,
      but_one_more: array(string)
    })
  | KebabCase({
      anotherList: array(string),
      camelCaseStringField: string,
      somethingElse: bool
    })

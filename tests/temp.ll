; ModuleID = 'thang_main'

define fastcc i32 @thang_main() {
entry:
  %x = alloca i32
  store i32 6, i32* %x
  %y = alloca i32
  store i32 7, i32* %y
  %"0" = load i32* %y
  %"1" = load i32* %x
  %"2" = mul nsw i32 %"0", %"1"
  store i32 %"2", i32* %x
  %"01" = load i32* %x
  ret i32 %"01"
}

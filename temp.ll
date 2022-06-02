; ModuleID = 'thang_main'

define fastcc i32 @thang_main() {
entry:
  %x = alloca i32
  store i32 4, i32* %x
  %0 = load i32* %x
  %1 = sub nsw i32 0, %0
  store i32 %1, i32* %x
  %2 = load i32* %x
  ret i32 %2
}

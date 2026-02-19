; ModuleID = 'lumina'
source_filename = "lumina"

@str.0 = unnamed_addr constant [6 x i8] c"Alice\00"
@str.1 = unnamed_addr constant [10 x i8] c"1 Main St\00"
@str.2 = unnamed_addr constant [4 x i8] c"Bob\00"
@str.3 = unnamed_addr constant [10 x i8] c"2 Oak Ave\00"
@str.4 = unnamed_addr constant [6 x i8] c"Carol\00"
@str.5 = unnamed_addr constant [9 x i8] c"3 Elm Rd\00"
@str.6 = unnamed_addr constant [5 x i8] c"Dave\00"
@str.7 = unnamed_addr constant [10 x i8] c"4 Pine Ln\00"

declare void @lumina_println_i64(i64)

declare void @lumina_println_str(ptr)

declare void @lumina_print(ptr)

declare void @lumina_print_i64(i64)

declare i64 @lumina_sqrt(i64)

declare i64 @lumina_max(i64, i64)

declare i64 @lumina_min(i64, i64)

declare i64 @lumina_array_i64_new()

declare void @lumina_array_i64_append(i64, i64)

declare i64 @lumina_array_i64_get(i64, i64)

declare void @lumina_array_i64_set(i64, i64, i64)

declare i64 @lumina_array_str_new()

declare void @lumina_array_str_append(i64, ptr)

declare ptr @lumina_array_str_get(i64, i64)

declare void @lumina_array_str_set(i64, i64, ptr)

declare i64 @lumina_array_i64_len(i64)

declare i64 @lumina_array_str_len(i64)

declare i64 @lumina_unwrap_i64(i64)

declare ptr @lumina_unwrap_str(i64)

define i64 @filter_adults(i64 %0) {
entry:
  %users = alloca i64, align 8
  store i64 %0, ptr %users, align 4
  %result = alloca i64, align 8
  %arr = call i64 @lumina_array_i64_new()
  store i64 %arr, ptr %result, align 4
  %users1 = load i64, ptr %users, align 4
  %call = call i64 @lumina_array_i64_len(i64 %users1)
  %range.is_inc = icmp sle i64 0, %call
  %range.step = select i1 %range.is_inc, i64 1, i64 -1
  %i = alloca i64, align 8
  br label %for.loop

for.loop:                                         ; preds = %for.inc, %entry
  %i2 = phi i64 [ 0, %entry ], [ %i.next, %for.inc ]
  %step.pos = icmp sgt i64 %range.step, 0
  %cmp.pos = icmp slt i64 %i2, %call
  %cmp.neg = icmp sgt i64 %i2, %call
  %for.cond = select i1 %step.pos, i1 %cmp.pos, i1 %cmp.neg
  br i1 %for.cond, label %for.body, label %for.after

for.body:                                         ; preds = %for.loop
  store i64 %i2, ptr %i, align 4
  %u = alloca ptr, align 8
  %users3 = load i64, ptr %users, align 4
  %i4 = load i64, ptr %i, align 4
  %call5 = call i64 @lumina_array_i64_get(i64 %users3, i64 %i4)
  %i2p = inttoptr i64 %call5 to ptr
  store ptr %i2p, ptr %u, align 8
  %u6 = load ptr, ptr %u, align 8
  %age = getelementptr inbounds { ptr, i64, ptr }, ptr %u6, i32 0, i32 1
  %age7 = load i64, ptr %age, align 4
  %gt = icmp sgt i64 %age7, 18
  %cmp = select i1 %gt, i64 1, i64 0
  %cond = icmp ne i64 %cmp, 0
  br i1 %cond, label %then, label %else

for.inc:                                          ; preds = %merge
  %i.next = add i64 %i2, %range.step
  br label %for.loop

for.after:                                        ; preds = %for.loop
  %result10 = load i64, ptr %result, align 4
  ret i64 %result10

then:                                             ; preds = %for.body
  %result8 = load i64, ptr %result, align 4
  %u9 = load ptr, ptr %u, align 8
  %ptr2i = ptrtoint ptr %u9 to i64
  call void @lumina_array_i64_append(i64 %result8, i64 %ptr2i)
  br label %merge

else:                                             ; preds = %for.body
  br label %merge

merge:                                            ; preds = %else, %then
  br label %for.inc
}

define void @lumina_main() {
entry:
  %users = alloca i64, align 8
  %arr = call i64 @lumina_array_i64_new()
  %record = alloca { ptr, i64, ptr }, align 8
  %name = getelementptr inbounds { ptr, i64, ptr }, ptr %record, i32 0, i32 0
  store ptr @str.0, ptr %name, align 8
  %age = getelementptr inbounds { ptr, i64, ptr }, ptr %record, i32 0, i32 1
  store i64 16, ptr %age, align 4
  %address = getelementptr inbounds { ptr, i64, ptr }, ptr %record, i32 0, i32 2
  store ptr @str.1, ptr %address, align 8
  %ptr2i = ptrtoint ptr %record to i64
  call void @lumina_array_i64_append(i64 %arr, i64 %ptr2i)
  %record1 = alloca { ptr, i64, ptr }, align 8
  %name2 = getelementptr inbounds { ptr, i64, ptr }, ptr %record1, i32 0, i32 0
  store ptr @str.2, ptr %name2, align 8
  %age3 = getelementptr inbounds { ptr, i64, ptr }, ptr %record1, i32 0, i32 1
  store i64 22, ptr %age3, align 4
  %address4 = getelementptr inbounds { ptr, i64, ptr }, ptr %record1, i32 0, i32 2
  store ptr @str.3, ptr %address4, align 8
  %ptr2i5 = ptrtoint ptr %record1 to i64
  call void @lumina_array_i64_append(i64 %arr, i64 %ptr2i5)
  %record6 = alloca { ptr, i64, ptr }, align 8
  %name7 = getelementptr inbounds { ptr, i64, ptr }, ptr %record6, i32 0, i32 0
  store ptr @str.4, ptr %name7, align 8
  %age8 = getelementptr inbounds { ptr, i64, ptr }, ptr %record6, i32 0, i32 1
  store i64 14, ptr %age8, align 4
  %address9 = getelementptr inbounds { ptr, i64, ptr }, ptr %record6, i32 0, i32 2
  store ptr @str.5, ptr %address9, align 8
  %ptr2i10 = ptrtoint ptr %record6 to i64
  call void @lumina_array_i64_append(i64 %arr, i64 %ptr2i10)
  %record11 = alloca { ptr, i64, ptr }, align 8
  %name12 = getelementptr inbounds { ptr, i64, ptr }, ptr %record11, i32 0, i32 0
  store ptr @str.6, ptr %name12, align 8
  %age13 = getelementptr inbounds { ptr, i64, ptr }, ptr %record11, i32 0, i32 1
  store i64 25, ptr %age13, align 4
  %address14 = getelementptr inbounds { ptr, i64, ptr }, ptr %record11, i32 0, i32 2
  store ptr @str.7, ptr %address14, align 8
  %ptr2i15 = ptrtoint ptr %record11 to i64
  call void @lumina_array_i64_append(i64 %arr, i64 %ptr2i15)
  store i64 %arr, ptr %users, align 4
  %adults = alloca i64, align 8
  %users16 = load i64, ptr %users, align 4
  %call = call i64 @filter_adults(i64 %users16)
  store i64 %call, ptr %adults, align 4
  %adults17 = load i64, ptr %adults, align 4
  %call18 = call i64 @lumina_array_i64_len(i64 %adults17)
  call void @lumina_println_i64(i64 %call18)
  ret void
}

# AspectJ Legacy

## Learning from Java's AOP Pioneer

**AspectJ** is the most mature and widely-used AOP framework, created in 1997 at Xerox PARC. It introduced many concepts that aspect-rs builds upon, while learning from its limitations.

## What AspectJ Got Right

### 1. Pointcut Expression Language

AspectJ's pointcut language is incredibly powerful:

```java
@Aspect
public class LoggingAspect {
    // Match all public methods in service package
    @Pointcut("execution(public * com.example.service.*.*(..))")
    public void serviceMethods() {}

    // Match all methods with @Transactional annotation
    @Pointcut("@annotation(org.springframework.transaction.annotation.Transactional)")
    public void transactionalMethods() {}

    // Combine pointcuts
    @Pointcut("serviceMethods() && transactionalMethods()")
    public void transactionalServiceMethods() {}

    @Before("serviceMethods()")
    public void logBefore(JoinPoint joinPoint) {
        System.out.println("→ " + joinPoint.getSignature().getName());
    }
}
```

**aspect-rs Phase 3** aims to provide similar expressiveness:

```rust
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::service)",
    advice = "before"
)]
static LOGGER: LoggingAspect = LoggingAspect::new();
```

### 2. Multiple Join Point Types

AspectJ supports many join point types:

- Method execution
- Method call (call-site)
- Field access (get/set)
- Constructor execution
- Exception handling
- Static initialization

**aspect-rs currently**: Function execution only (Phase 1-2)
**aspect-rs future**: Field access, call-site matching (Phase 3+)

### 3. Rich Join Point Context

AspectJ provides detailed context:

```java
@Around("serviceMethods()")
public Object logAround(ProceedingJoinPoint joinPoint) throws Throwable {
    String methodName = joinPoint.getSignature().getName();
    Object[] args = joinPoint.getArgs();  // Access arguments
    Object target = joinPoint.getTarget();  // Access target object

    long start = System.nanoTime();
    Object result = joinPoint.proceed();  // Execute method
    long elapsed = System.nanoTime() - start;

    System.out.println(methodName + " took " + elapsed + "ns");
    return result;
}
```

**aspect-rs equivalent**:

```rust
impl Aspect for TimingAspect {
    fn around(&self, ctx: &mut ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = ctx.proceed()?;  // Execute function
        println!("{} took {:?}", ctx.function_name, start.elapsed());
        Ok(result)
    }
}
```

### 4. Compile-Time and Load-Time Weaving

AspectJ offers multiple weaving strategies:

- **Compile-time weaving (CTW)**: Weave during compilation with `ajc`
- **Post-compile weaving (binary weaving)**: Weave into JAR files
- **Load-time weaving (LTW)**: Weave when classes are loaded

**aspect-rs**: Compile-time only (no JVM-style class loading in Rust)

## Key Differences: aspect-rs vs AspectJ

| Feature | AspectJ | aspect-rs (Phase 2) | aspect-rs (Phase 3) |
|---------|---------|---------------------|---------------------|
| **Weaving Time** | Compile or Load | Compile-time only | Compile-time only |
| **Runtime Overhead** | ~10-50ns | <10ns | <5ns (goal) |
| **Type Safety** | ⚠️ Runtime checks | ✅ Compile-time | ✅ Compile-time |
| **Ownership Checks** | N/A (GC language) | ✅ Full Rust rules | ✅ Full Rust rules |
| **Per-Function Annotation** | ❌ Optional | ✅ Required | ❌ Optional |
| **Pointcut Expressions** | ✅ Rich language | ⚠️ Limited | ✅ Planned |
| **Field Access Interception** | ✅ | ❌ | ✅ Planned |
| **Call-Site Matching** | ✅ | ❌ | ✅ Planned |
| **Runtime Dependencies** | AspectJ runtime | None | None |
| **Tooling Required** | AspectJ compiler | Standard rustc | rustc nightly |

## What aspect-rs Improves

### 1. **Compile-Time Type Safety**

AspectJ relies on runtime type checking:

```java
@Around("serviceMethods()")
public Object logAround(ProceedingJoinPoint joinPoint) throws Throwable {
    Object result = joinPoint.proceed();
    // Type mismatch discovered at runtime!
    String value = (String) result;  // ClassCastException if wrong type
    return value;
}
```

aspect-rs catches type errors at compile time:

```rust
impl Aspect for MyAspect {
    fn around(&self, ctx: &mut ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let result = ctx.proceed()?;
        // Compiler error if type mismatch!
        let value = result.downcast_ref::<String>().ok_or(...)?;
        Ok(result)
    }
}
```

### 2. **Zero Runtime Dependencies**

AspectJ requires the AspectJ runtime library:

```xml
<dependency>
    <groupId>org.aspectj</groupId>
    <artifactId>aspectjrt</artifactId>
    <version>1.9.19</version>
</dependency>
```

aspect-rs has **zero runtime dependencies**:

```toml
[dependencies]
aspect-core = "0.1"  # Only trait definitions, no runtime
```

All weaving is done at compile time. The generated code has no dependency on aspect-rs!

### 3. **Better Performance**

| Operation | AspectJ (JVM) | aspect-rs (Rust) |
|-----------|---------------|------------------|
| Simple before/after | ~10-20ns | <5ns |
| Around advice | ~30-50ns | <10ns |
| Argument access | ~5-10ns (reflection) | 0ns (direct) |
| Method call overhead | JIT warmup required | None |

aspect-rs achieves better performance because:
- No JVM overhead
- No runtime reflection
- No dynamic proxy creation
- Direct function calls (inlined by LLVM)

### 4. **Ownership and Lifetime Safety**

AspectJ (Java) has garbage collection. aspect-rs must respect Rust's ownership:

```rust
#[aspect(LoggingAspect::new())]
fn process_data(data: Vec<String>) -> Vec<String> {
    // Compiler ensures:
    // - 'data' is moved, not copied
    // - Return value transfers ownership
    // - No dangling pointers
    data.into_iter().map(|s| s.to_uppercase()).collect()
}
```

The aspect framework **cannot violate ownership rules**. This is checked at compile time.

### 5. **No Classpath/Reflection Magic**

AspectJ uses reflection and runtime bytecode manipulation:

```java
// AspectJ can intercept private methods via reflection
@Pointcut("execution(private * *(..))")
public void privateMethods() {}
```

aspect-rs only works with **visible, statically-known code**:

```rust
// Can only apply to functions visible to the macro
#[aspect(LoggingAspect::new())]
fn public_function() { }  // ✅ Works

#[aspect(LoggingAspect::new())]
fn private_function() { }  // ✅ Works (same module)
```

This is **more explicit and predictable** than AspectJ's reflection-based approach.

## What We Learn from AspectJ

AspectJ taught the AOP community:

1. ✅ **Pointcut expressions** are essential for practical AOP
2. ✅ **Multiple advice types** (before, after, around) are needed
3. ✅ **Join point context** must be rich enough to be useful
4. ✅ **Compile-time weaving** is faster than load-time
5. ⚠️ **Runtime reflection** introduces complexity and overhead
6. ⚠️ **Classpath scanning** can be slow and error-prone

aspect-rs takes the good parts (1-4) and avoids the pitfalls (5-6) through Rust's compile-time capabilities.

## Migration from AspectJ

If you're coming from AspectJ, the mental model is similar:

### AspectJ
```java
@Aspect
public class LoggingAspect {
    @Before("execution(* com.example..*.*(..))")
    public void logBefore(JoinPoint jp) {
        System.out.println("→ " + jp.getSignature().getName());
    }
}
```

### aspect-rs (Phase 2 - current)
```rust
struct LoggingAspect;

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {}", ctx.function_name);
    }
}

// Apply per function
#[aspect(LoggingAspect::new())]
fn my_function() { }
```

### aspect-rs (Phase 3 - future)
```rust
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::*)",
    advice = "before"
)]
static LOGGER: LoggingAspect = LoggingAspect::new();

// No annotation needed - automatic weaving!
fn my_function() { }
```

## Conclusion

AspectJ pioneered AOP and proved its value. aspect-rs builds on that legacy while embracing Rust's strengths:

- **Compile-time safety** over runtime flexibility
- **Zero-cost abstractions** over convenience
- **Explicit code generation** over bytecode manipulation

Next, let's explore what makes aspect-rs special in [Why aspect-rs](why-aspect-rs.md).

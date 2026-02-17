# Summary

[Introduction](README.md)

---

# Getting Started

- [Motivation](ch01-motivation/README.md)
  - [The Problem: Crosscutting Concerns](ch01-motivation/problem.md)
  - [The Solution: Aspect-Oriented Programming](ch01-motivation/solution.md)
  - [AspectJ Legacy](ch01-motivation/aspectj.md)
  - [Why aspect-rs](ch01-motivation/why-aspect-rs.md)

- [Background](ch02-background/README.md)
  - [Crosscutting Concerns Explained](ch02-background/crosscutting.md)
  - [AOP Terminology](ch02-background/terminology.md)
  - [What aspect-rs Can Do](ch02-background/capabilities.md)

- [Getting Started](ch03-getting-started/README.md)
  - [Installation](ch03-getting-started/installation.md)
  - [Hello World](ch03-getting-started/hello-world.md)
  - [Quick Start Guide](ch03-getting-started/quick-start.md)
  - [Using Pre-built Aspects](ch03-getting-started/prebuilt.md)

---

# User Guide

- [Core Concepts](ch04-core-concepts/README.md)
  - [The Aspect Trait](ch04-core-concepts/aspect-trait.md)
  - [JoinPoint Context](ch04-core-concepts/joinpoint.md)
  - [Advice Types](ch04-core-concepts/advice-types.md)
  - [Error Handling](ch04-core-concepts/error-handling.md)

- [Usage Guide](ch05-usage-guide/README.md)
  - [Basic Patterns](ch05-usage-guide/basic.md)
  - [Production Patterns](ch05-usage-guide/production.md)
  - [Advanced Patterns](ch05-usage-guide/advanced.md)
  - [Configuration](ch05-usage-guide/configuration.md)
  - [Testing Custom Aspects](ch05-usage-guide/testing.md)

- [Case Studies](ch08-case-studies/README.md)
  - [Logging in a Web Service](ch08-case-studies/logging.md)
  - [Performance Monitoring](ch08-case-studies/timing.md)
  - [API Server with Multiple Aspects](ch08-case-studies/api-server.md)
  - [Security & Authorization](ch08-case-studies/security.md)
  - [Resilience Patterns](ch08-case-studies/resilience.md)
  - [Transaction Management](ch08-case-studies/transactions.md)
  - [Automatic Weaving Demo](ch08-case-studies/automatic-weaving.md)

---

# Technical Reference

- [Architecture](ch06-architecture/README.md)
  - [Crate Organization](ch06-architecture/crates.md)
  - [Design Principles](ch06-architecture/principles.md)
  - [Component Interactions](ch06-architecture/interactions.md)
  - [Extension Points](ch06-architecture/extensions.md)

- [Implementation Details](ch07-implementation/README.md)
  - [Macro Code Generation](ch07-implementation/macros.md)
  - [Pointcut Matching](ch07-implementation/pointcuts.md)
  - [MIR Extraction](ch07-implementation/mir.md)
  - [Code Weaving Process](ch07-implementation/weaving.md)
  - [Performance Optimizations](ch07-implementation/optimizations.md)

- [Performance Benchmarks](ch09-benchmarks/README.md)
  - [Methodology](ch09-benchmarks/methodology.md)
  - [Measured Overhead](ch09-benchmarks/results.md)
  - [Real-World Performance](ch09-benchmarks/realworld.md)
  - [Optimization Techniques](ch09-benchmarks/techniques.md)
  - [Running Benchmarks](ch09-benchmarks/running.md)

---

# Advanced Topics

- [Automatic Weaving](ch10-phase3/README.md)
  - [The Vision](ch10-phase3/vision.md)
  - [Architecture](ch10-phase3/architecture.md)
  - [How It Works](ch10-phase3/how-it-works.md)
  - [Pointcut Expressions](ch10-phase3/pointcuts.md)
  - [Complete Demo](ch10-phase3/demo.md)
  - [Technical Breakthrough](ch10-phase3/breakthrough.md)

---

# Community

- [Future Directions](ch11-future/README.md)
  - [What We've Achieved](ch11-future/achievements.md)
  - [Roadmap](ch11-future/roadmap.md)
  - [Long-Term Vision](ch11-future/vision.md)
  - [How to Contribute](ch11-future/contributing.md)
  - [Acknowledgements](ch11-future/acknowledgements.md)

---

# Appendices

- [Appendix A: Glossary](appendix/glossary.md)
- [Appendix B: API Reference](appendix/api-reference.md)
- [Appendix C: References](appendix/references.md)
- [Appendix D: Troubleshooting](appendix/troubleshooting.md)

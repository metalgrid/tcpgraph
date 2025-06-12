# TCPGraph Documentation

This directory contains comprehensive documentation for the TCPGraph project, covering architecture, usage, development, and lessons learned.

## Documentation Structure

### User Documentation
- **[User Guide](user-guide.md)**: Complete guide for end users
  - Quick start and basic usage
  - Command-line options and examples
  - Interface selection and PCAP filters
  - Troubleshooting common issues

### Technical Documentation
- **[Architecture](architecture.md)**: System design and component overview
  - Core components and data flow
  - Threading model and communication
  - Key design decisions and trade-offs

- **[Traffic Direction Detection](traffic-direction.md)**: MAC address-based direction analysis
  - Algorithm explanation and edge cases
  - Router and multi-interface scenarios
  - Performance considerations

- **[Bandwidth Accuracy](bandwidth-accuracy.md)**: Measurement methodology and accuracy
  - Header overhead analysis
  - Payload-only vs. standard mode comparison
  - Smoothing algorithms and validation

### Development Documentation
- **[Development Guide](development.md)**: For contributors and maintainers
  - Project structure and build requirements
  - Code architecture and key algorithms
  - Testing strategy and debugging tips
  - Release process and future roadmap

- **[Lessons Learned](lessons-learned.md)**: Project evolution and insights
  - Major design decisions and their rationale
  - Technical challenges and solutions
  - User experience insights
  - Recommendations for similar projects

## Quick Navigation

### For New Users
Start with the [User Guide](user-guide.md) for installation and basic usage examples.

### For System Administrators
See the [Traffic Direction Detection](traffic-direction.md) guide for router and infrastructure monitoring.

### For Speed Test Comparisons
Read the [Bandwidth Accuracy](bandwidth-accuracy.md) documentation to understand measurement differences.

### For Developers
Begin with the [Architecture](architecture.md) overview, then dive into the [Development Guide](development.md).

### For Project Maintainers
Review [Lessons Learned](lessons-learned.md) for insights on project evolution and decision rationale.

## Documentation Philosophy

This documentation follows several key principles:

1. **Multi-layered**: Different audiences need different levels of detail
2. **Example-driven**: Show concrete usage patterns and scenarios
3. **Problem-focused**: Address real user pain points and questions
4. **Evolution-aware**: Document not just what exists, but why and how it evolved

## Contributing to Documentation

When contributing to TCPGraph, please also update relevant documentation:

- **Code changes**: Update architecture.md and development.md
- **New features**: Add examples to user-guide.md
- **Bug fixes**: Update troubleshooting sections
- **Performance improvements**: Document in lessons-learned.md

## Documentation Maintenance

This documentation should be reviewed and updated with each major release to ensure accuracy and completeness. Pay special attention to:

- Command-line option changes
- New feature documentation
- Updated installation requirements
- Changed system requirements or dependencies
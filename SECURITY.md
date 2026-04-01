# Security Policy

## Supported Versions

Security updates are provided for the following versions:

| Version | Status          | Security Updates |
|---------|-----------------|------------------|
| 0.3.x   | Current         | Yes              |
| 0.2.x   | Maintenance     | Yes (critical only) |
| < 0.2.0 | End of Life     | No               |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly.

### How to Report

**Email**: security@tairitsu.dev

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Proof of concept (if available)

### What to Expect

1. **Confirmation**: We'll acknowledge receipt within 24 hours
2. **Assessment**: We'll investigate and assess severity within 3 business days
3. **Resolution**: We'll develop and test a fix
4. **Disclosure**: We'll coordinate disclosure with you

### Disclosure Policy

- Critical/High severity: Fix within 7 days, disclose after fix deployed
- Medium severity: Fix within 30 days, disclose after fix deployed
- Low severity: Fix in next release, public disclosure

## Security Best Practices

### For Application Developers

1. **Keep Dependencies Updated**
   ```bash
   cargo update
   ```

2. **Enable Content Security Policy**
   ```toml
   [packager.security]
   csp = "default-src 'self'; script-src 'self' 'unsafe-inline'"
   ```

3. **Sanitize User Input**
   ```rust
   use tairitsu_web::sanitize;

   let safe_html = sanitize_html(user_input);
   ```

4. **Use HTTPS in Production**
   ```toml
   [packager.server]
   force_https = true
   ```

### For Deployments

1. **Use Verified Container Images**
2. **Enable Subresource Integrity (SRI)**
3. **Implement Rate Limiting**
4. **Regular Security Audits**

## Security Features

### Built-in Protections

- **XSS Prevention**: Automatic HTML escaping
- **CSP Support**: Content-Security-Policy headers
- **SRI**: Subresource Integrity for external resources
- **Safe SVG**: Sanitized SVG embedding

### WASM Sandboxing

Tairitsu runs WebAssembly in a browser sandbox:
- Memory isolation
- No direct DOM access (via WIT)
- Capability-based security

## Security Audits

| Date       | Auditor     | Scope            | Report |
|------------|-------------|------------------|--------|
| TBD        | TBD         | Full Framework   | TBD    |

## Coordinated Disclosure

We follow responsible disclosure practices:
- Private vulnerability reporting
- Time to fix before disclosure
- Credit to reporters
- Clear communication

## Security Contacts

- **Vulnerabilities**: security@tairitsu.dev
- **Security Questions**: security@tairitsu.dev
- **Enterprise Security**: enterprise@tairitsu.dev

## Related Resources

- [Enterprise Support](docs/en-US/enterprise/support.md)
- [Privacy Policy](PRIVACY.md)
- [Terms of Service](TERMS.md)

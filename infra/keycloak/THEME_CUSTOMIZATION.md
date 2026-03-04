# Keycloak Theme Customization Guide

## Overview

This guide explains how to customize the look and feel of your Keycloak login pages, account console, and emails for the can-bank-x project.

## Theme Structure

The custom theme is located at `infra/keycloak/themes/can-bank-x/`:

```
can-bank-x/
├── login/                    # Login pages theme
│   ├── theme.properties      # Theme configuration
│   └── resources/
│       ├── css/
│       │   └── custom.css    # Custom styles
│       ├── img/              # Images and logos
│       └── js/               # Custom JavaScript
├── account/                  # Account console theme (optional)
└── email/                    # Email templates (optional)
```

## Current Customization

The theme uses:
- **Primary Color**: `#0066cc` (Blue)
- **Secondary Color**: `#004080` (Dark Blue)
- **Accent Color**: `#00a3e0` (Light Blue)
- Modern gradient background
- Rounded corners and shadows
- Smooth transitions and hover effects

## How to Apply Changes

### 1. Modify CSS Styles

Edit `infra/keycloak/themes/can-bank-x/login/resources/css/custom.css`:

```css
/* Change primary color */
:root {
  --primary-color: #YOUR_COLOR;
  --secondary-color: #YOUR_DARKER_COLOR;
}
```

### 2. Add Your Logo

1. Place your logo in `infra/keycloak/themes/can-bank-x/login/resources/img/logo.png`
2. Create or modify `infra/keycloak/themes/can-bank-x/login/resources/css/custom.css`:

```css
/* Add logo to header */
#kc-header-wrapper::before {
  content: '';
  display: block;
  width: 200px;
  height: 80px;
  margin: 0 auto 20px;
  background: url('../img/logo.png') no-repeat center;
  background-size: contain;
}
```

### 3. Customize HTML Templates (Advanced)

To fully customize HTML structure, copy templates from the base theme:

```bash
# From inside Keycloak container
docker exec can-bank-x-keycloak-1 cp -r /opt/keycloak/lib/lib/main/org.keycloak.keycloak-themes-*.jar /tmp/
# Extract and copy templates to your theme directory
```

Common templates to customize:
- `login.ftl` - Main login page
- `register.ftl` - Registration page
- `login-reset-password.ftl` - Password reset
- `login-update-profile.ftl` - Profile update
- `info.ftl` - Info pages
- `error.ftl` - Error pages

Place custom templates in: `infra/keycloak/themes/can-bank-x/login/`

### 4. Add Custom Fonts

```css
/* In custom.css */
@font-face {
  font-family: 'YourFont';
  src: url('../fonts/your-font.woff2') format('woff2');
}

body {
  font-family: 'YourFont', sans-serif;
}
```

Place fonts in: `infra/keycloak/themes/can-bank-x/login/resources/fonts/`

### 5. Customize Messages/Translations

Create `infra/keycloak/themes/can-bank-x/login/messages/messages_en.properties`:

```properties
# Custom login page text
loginTitle=Welcome to Can-Bank-X
loginAccountTitle=Sign in to your account
doLogIn=Sign In
registerTitle=Create your account
doRegister=Sign Up
```

For French: `messages_fr.properties`

## Applying Theme Changes

### Method 1: Restart Keycloak (Fastest for CSS/Images)

```bash
docker compose restart keycloak
```

Wait ~20 seconds for Keycloak to start, then **clear your browser cache** (Ctrl+F5).

### Method 2: Complete Rebuild (For Template Changes)

```bash
docker compose rm -sf keycloak
docker compose up -d keycloak
```

Wait ~30 seconds for full initialization.

### Method 3: Disable Theme Caching (Development)

In `docker-compose.yml`, add to Keycloak environment:

```yaml
environment:
  KEYCLOAK_ADMIN: ${KEYCLOAK_ADMIN}
  KEYCLOAK_ADMIN_PASSWORD: ${KEYCLOAK_ADMIN_PASSWORD}
  KC_SPI_THEME_STATIC_MAX_AGE: -1
  KC_SPI_THEME_CACHE_THEMES: false
  KC_SPI_THEME_CACHE_TEMPLATES: false
```

Then restart: `docker compose up -d keycloak`

## Testing Your Theme

1. **Access Login Page**: 
   ```
   http://localhost:8080/auth/realms/can-bank-x/protocol/openid-connect/auth?client_id=can-bank-x-api&redirect_uri=http://localhost:8083/callback&response_type=code
   ```

2. **Access Account Console**:
   ```
   http://localhost:8082/realms/can-bank-x/account
   ```

3. **Test in Different Browsers**: Chrome, Firefox, Edge, Safari

4. **Test Mobile Responsiveness**: Use browser dev tools (F12 → Device Toolbar)

## Theme Configuration Options

In `theme.properties`:

```properties
# Inherit from base theme
parent=keycloak
import=common/keycloak

# Custom stylesheets (loaded in order)
styles=css/login.css css/custom.css css/mobile.css

# Custom scripts
scripts=js/custom.js

# Supported languages
locales=en,fr,es

# Meta tags
meta=viewport="width=device-width, initial-scale=1"
```

## Common Customizations

### Change Background

```css
body.login-pf {
  background: url('../img/background.jpg') center/cover no-repeat;
  /* OR */
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}
```

### Modify Button Styles

```css
#kc-login {
  background: linear-gradient(45deg, #667eea, #764ba2);
  border-radius: 25px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}
```

### Custom Login Card

```css
#kc-content-wrapper {
  max-width: 450px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

### Add Animated Elements

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

#kc-content-wrapper {
  animation: fadeIn 0.5s ease-out;
}
```

## Troubleshooting

### Theme Not Showing

1. **Check realm configuration**: Verify `loginTheme: "can-bank-x"` in realm JSON
2. **Check theme directory**: Ensure `/opt/keycloak/themes/can-bank-x` is mounted
3. **Check logs**: `docker compose logs keycloak | grep -i theme`
4. **Verify theme properties**: Check `theme.properties` syntax

### CSS Changes Not Applying

1. **Clear browser cache**: Ctrl+F5 or Ctrl+Shift+R
2. **Clear Keycloak cache**: Restart container
3. **Check CSS syntax**: Look for errors in browser console (F12)
4. **Verify file path**: Check CSS file path in `theme.properties`

### Logo Not Displaying

1. **Check image path**: Verify path in CSS is correct (`../img/logo.png`)
2. **Check file permissions**: Ensure image file is readable
3. **Check image format**: Use PNG, JPG, or SVG
4. **Verify volume mount**: Confirm theme directory is mounted in docker-compose.yml

## Advanced: Email Theme

To customize emails (verification, password reset):

1. Create `infra/keycloak/themes/can-bank-x/email/`
2. Add `theme.properties`:
   ```properties
   parent=keycloak
   ```
3. Create templates:
   - `html/email-verification.ftl`
   - `html/password-reset.ftl`
   - `text/` (plain text versions)

4. Update realm: `"emailTheme": "can-bank-x"`

## Resources

- [Keycloak Themes Documentation](https://www.keycloak.org/docs/latest/server_development/#_themes)
- [FreeMarker Template Documentation](https://freemarker.apache.org/docs/)
- [PatternFly CSS Framework](https://www.patternfly.org/) (used by Keycloak)

## Quick Reference Commands

```bash
# Restart Keycloak
docker compose restart keycloak

# Rebuild Keycloak with theme
docker compose rm -sf keycloak && docker compose up -d keycloak

# View Keycloak logs
docker compose logs -f keycloak

# Access Keycloak container
docker exec -it can-bank-x-keycloak-1 bash

# List installed themes
docker exec can-bank-x-keycloak-1 ls -la /opt/keycloak/themes/

# Clear browser cache
Ctrl+F5 (Windows/Linux) or Cmd+Shift+R (Mac)
```

## Color Scheme Examples

### Professional Blue (Current)
```css
--primary-color: #0066cc;
--secondary-color: #004080;
--accent-color: #00a3e0;
```

### Banking Green
```css
--primary-color: #047857;
--secondary-color: #065f46;
--accent-color: #10b981;
```

### Modern Purple
```css
--primary-color: #7c3aed;
--secondary-color: #5b21b6;
--accent-color: #a78bfa;
```

### Corporate Gray
```css
--primary-color: #374151;
--secondary-color: #1f2937;
--accent-color: #6b7280;
```

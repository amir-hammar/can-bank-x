(() => {
  const HOME_ORIGIN_KEY = 'cbx.home.origin';
  const HOME_ORIGIN_COOKIE = 'cbx_home_origin';
  const REGISTER_DRAFT_KEY = 'cbx.register.draft';
  const REGISTER_DRAFT_COOKIE = 'cbx_register_draft';

  const EYE_ICON = `<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7S1 12 1 12Z"/><circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2"/></svg>`;
  const EYE_OFF_ICON = `<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="m3 3 18 18M10.58 10.58A3 3 0 0 0 12 15a3 3 0 0 0 2.42-4.42M9.88 5.09A10.94 10.94 0 0 1 12 5c7 0 11 7 11 7a21.8 21.8 0 0 1-5.07 5.94M6.61 6.61C3.71 8.58 1.99 12 1.99 12a21.8 21.8 0 0 0 9.24 6.54"/></svg>`;
  const EMAIL_REGEX = /^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$/;
  const FULL_NAME_REGEX = /^[\p{L}\s'-]+$/u;
  const POSTAL_REGEX = /^[A-Za-z]\d[A-Za-z]\d[A-Za-z]\d$/;
  const PASSWORD_SPECIAL_REGEX = /[^A-Za-z0-9]/;

  const normalizeSpaces = (value) => value.split(/\s+/).filter(Boolean).join(' ').trim();
  const compactPostalCode = (value) => value.replace(/[\s-]+/g, '');
  const compactNas = (value) => value.replace(/\s+/g, '');

  const safeOrigin = (value) => {
    if (!value) {
      return null;
    }

    try {
      return new URL(value, window.location.href).origin;
    } catch {
      return null;
    }
  };

  const redirectOriginFromUrl = (value) => {
    if (!value) {
      return null;
    }

    try {
      const parsed = new URL(value, window.location.href);
      return safeOrigin(parsed.searchParams.get('redirect_uri'));
    } catch {
      return null;
    }
  };

  const writeHomeOriginCookie = (origin) => {
    if (!origin) {
      return;
    }
    document.cookie = `${HOME_ORIGIN_COOKIE}=${encodeURIComponent(origin)}; Path=/; SameSite=Lax`;
  };

  const readHomeOriginCookie = () => {
    const pairs = document.cookie ? document.cookie.split('; ') : [];
    const found = pairs.find((entry) => entry.startsWith(`${HOME_ORIGIN_COOKIE}=`));
    if (!found) {
      return null;
    }
    const raw = found.substring(HOME_ORIGIN_COOKIE.length + 1);
    try {
      return decodeURIComponent(raw);
    } catch {
      return raw;
    }
  };

  const writeDraftCookie = (value) => {
    if (!value) {
      return;
    }
    document.cookie = `${REGISTER_DRAFT_COOKIE}=${encodeURIComponent(value)}; Path=/; SameSite=Lax`;
  };

  const readDraftCookie = () => {
    const pairs = document.cookie ? document.cookie.split('; ') : [];
    const found = pairs.find((entry) => entry.startsWith(`${REGISTER_DRAFT_COOKIE}=`));
    if (!found) {
      return null;
    }
    try {
      return decodeURIComponent(found.substring(REGISTER_DRAFT_COOKIE.length + 1));
    } catch {
      return null;
    }
  };

  const resolveHomeHref = () => {
    const redirectOrigin = redirectOriginFromUrl(window.location.href);
    if (redirectOrigin && redirectOrigin !== window.location.origin) {
      sessionStorage.setItem(HOME_ORIGIN_KEY, redirectOrigin);
      writeHomeOriginCookie(redirectOrigin);
      return `${redirectOrigin}/`;
    }

    const cachedOrigin = safeOrigin(sessionStorage.getItem(HOME_ORIGIN_KEY));
    if (cachedOrigin && cachedOrigin !== window.location.origin) {
      return `${cachedOrigin}/`;
    }

    const cookieOrigin = safeOrigin(readHomeOriginCookie());
    if (cookieOrigin && cookieOrigin !== window.location.origin) {
      return `${cookieOrigin}/`;
    }

    const referrerRedirectOrigin = redirectOriginFromUrl(document.referrer);
    if (referrerRedirectOrigin && referrerRedirectOrigin !== window.location.origin) {
      sessionStorage.setItem(HOME_ORIGIN_KEY, referrerRedirectOrigin);
      writeHomeOriginCookie(referrerRedirectOrigin);
      return `${referrerRedirectOrigin}/`;
    }

    const referrerOrigin = safeOrigin(document.referrer);
    if (referrerOrigin && referrerOrigin !== window.location.origin) {
      sessionStorage.setItem(HOME_ORIGIN_KEY, referrerOrigin);
      writeHomeOriginCookie(referrerOrigin);
      return `${referrerOrigin}/`;
    }

    return '/';
  };

  const addHomeButton = () => {
    if (document.querySelector('.cbx-home-button')) {
      return;
    }

    const button = document.createElement('a');
    button.className = 'cbx-home-button';
    button.href = resolveHomeHref();
    button.setAttribute('aria-label', 'Go to home');
    button.innerHTML = `
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <path fill="currentColor" d="M12 3.2 2.8 10.7a1 1 0 0 0 1.3 1.54l1.4-1.16V19a2 2 0 0 0 2 2h3.8a1 1 0 0 0 1-1v-4.3h1.4V20a1 1 0 0 0 1 1h3.8a2 2 0 0 0 2-2v-7.92l1.4 1.16a1 1 0 1 0 1.28-1.54L12 3.2z"/>
      </svg>
      <span>Home</span>
    `;

    document.body.appendChild(button);
  };

  const loadRegisterDraft = () => {
    let sessionDraft = {};
    try {
      const raw = sessionStorage.getItem(REGISTER_DRAFT_KEY);
      sessionDraft = raw ? JSON.parse(raw) : {};
    } catch {
      sessionDraft = {};
    }

    try {
      const cookieRaw = readDraftCookie();
      const cookieDraft = cookieRaw ? JSON.parse(cookieRaw) : {};
      return { ...cookieDraft, ...sessionDraft };
    } catch {
      return sessionDraft;
    }
  };

  const saveRegisterDraft = (draft) => {
    const serialized = JSON.stringify(draft);
    try {
      sessionStorage.setItem(REGISTER_DRAFT_KEY, serialized);
    } catch {
      // Ignore storage quota/unavailable errors.
    }
    writeDraftCookie(serialized);
  };

  const updateRegisterDraftFromForm = (registerForm) => {
    if (!registerForm) {
      return;
    }

    const pick = (selector) => registerForm.querySelector(selector)?.value?.trim() || '';
    const draft = {
      fullName: pick('#cbx-fullName'),
      username: pick('#username'),
      email: pick('#email'),
      street: pick('#cbx-street'),
      city: pick('#cbx-city'),
      province: pick('#cbx-province'),
      postalCode: pick('#cbx-postalCode'),
      country: pick('#cbx-country'),
      nas: pick('#cbx-nas')
    };

    saveRegisterDraft(draft);
  };

  const restoreRegisterDraftToForm = (registerForm) => {
    const draft = loadRegisterDraft();
    const setValueIfEmpty = (selector, value) => {
      if (!value) {
        return;
      }
      const input = registerForm.querySelector(selector);
      if (!input) {
        return;
      }
      if (typeof input.value === 'string' && input.value.trim() !== '') {
        return;
      }
      input.value = value;
    };

    setValueIfEmpty('#cbx-fullName', draft.fullName);
    setValueIfEmpty('#username', draft.username);
    setValueIfEmpty('#email', draft.email);
    setValueIfEmpty('#cbx-street', draft.street);
    setValueIfEmpty('#cbx-city', draft.city);
    setValueIfEmpty('#cbx-province', draft.province);
    setValueIfEmpty('#cbx-postalCode', draft.postalCode);
    setValueIfEmpty('#cbx-country', draft.country);
    setValueIfEmpty('#cbx-nas', draft.nas);
  };

  const redirectIfAlreadyAuthenticatedForRegistration = () => {
    if (!window.location.pathname.includes('/login-actions/')) {
      return;
    }

    let redirected = false;

    const tryRedirect = () => {
      if (redirected) {
        return;
      }

      const pageText = (document.body?.innerText || '').toLowerCase();
      const isConflictPage =
        pageText.includes('already authenticated as different user') ||
        pageText.includes('please sign out first') ||
        pageText.includes('different user');

      if (!isConflictPage) {
        return;
      }

      const target = resolveHomeHref();
      if (!target) {
        return;
      }

      redirected = true;
      document.documentElement.style.visibility = 'hidden';
      window.location.replace(target);
    };

    tryRedirect();
    setTimeout(tryRedirect, 100);
    setTimeout(tryRedirect, 300);
    setTimeout(tryRedirect, 700);

    const observer = new MutationObserver(() => {
      tryRedirect();
      if (redirected) {
        observer.disconnect();
      }
    });

    observer.observe(document.body, { childList: true, subtree: true, characterData: true });
    setTimeout(() => observer.disconnect(), 5000);
  };

  const applyAuthLayoutScaffold = () => {
    const header = document.querySelector('.login-pf-header');
    const pageTitle = document.getElementById('kc-page-title');
    if (!header || !pageTitle) {
      return;
    }

    const registerForm = document.getElementById('kc-register-form');
    const isRegister = Boolean(registerForm);

    if (!header.querySelector('.cbx-brand-row')) {
      const brandRow = document.createElement('div');
      brandRow.className = 'cbx-brand-row';
      brandRow.innerHTML = `
        <svg class="cbx-brand-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
          <path fill="currentColor" d="M3 7.5 7.5 12l4.5-5 4.5 5L21 7.5V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7.5Zm2 .6V18h14V8.1l-2.5 2.5a1 1 0 0 1-1.46-.04L12 7.4l-3.04 3.16a1 1 0 0 1-1.46.04L5 8.1Z"/>
        </svg>
        <span>CanBankX</span>
      `;
      header.insertBefore(brandRow, header.firstChild);
    }

    if (!header.querySelector('.cbx-page-subtitle')) {
      const subtitle = document.createElement('p');
      subtitle.className = 'cbx-page-subtitle';
      subtitle.textContent = isRegister
        ? 'Provide your details to create your account.'
        : 'Use your credentials to access your account.';
      pageTitle.insertAdjacentElement('afterend', subtitle);
    }
  };

  const removeFieldGroup = (input) => {
    if (!input) {
      return;
    }
    const group = input.closest('.form-group') || input.parentElement;
    if (group) {
      group.remove();
    } else {
      input.remove();
    }
  };

  const normalizeRealmManagedRegisterFields = (registerForm) => {
    const builtInPassword = registerForm.querySelector('input[name="password"][type="password"]');
    const duplicatePasswordInputs = Array.from(registerForm.querySelectorAll('input[name="password"]'))
      .filter((input) => input !== builtInPassword);
    duplicatePasswordInputs.forEach(removeFieldGroup);

    // If the realm profile defines its own confirmation field, it duplicates Keycloak's built-in one.
    const profileConfirmPassword = registerForm.querySelector('input[name="confirmPassword"]');
    removeFieldGroup(profileConfirmPassword);

    [
      'input[name="fullName"]',
      'input[name="street"]',
      'input[name="city"]',
      'input[name="postalCode"]',
      'input[name="province"]',
      'select[name="province"]',
      'input[name="country"]',
      'input[name="nas"]',
      'input[name="user.attributes.fullName"]',
      'input[name="user.attributes.street"]',
      'input[name="user.attributes.city"]',
      'input[name="user.attributes.postalCode"]',
      'input[name="user.attributes.province"]',
      'select[name="user.attributes.province"]',
      'input[name="user.attributes.country"]',
      'input[name="user.attributes.nas"]'
    ].forEach((selector) => {
      registerForm.querySelectorAll(selector).forEach(removeFieldGroup);
    });
  };

  const enhancePasswordToggleIcons = () => {
    const buttons = document.querySelectorAll('button[aria-controls="password"], button[aria-controls="password-confirm"], button[data-password-toggle]');

    buttons.forEach((btn) => {
      const targetId = btn.getAttribute('aria-controls');
      const targetInput = targetId ? document.getElementById(targetId) : null;
      const isVisible = targetInput?.type === 'text';

      btn.classList.add('cbx-password-toggle');
      btn.type = 'button';
      btn.setAttribute('aria-label', isVisible ? 'Hide password' : 'Show password');
      btn.innerHTML = isVisible ? EYE_OFF_ICON : EYE_ICON;
    });
  };

  const enhanceRegisterFields = () => {
    const registerForm = document.getElementById('kc-register-form');
    if (!registerForm || registerForm.querySelector('.cbx-register-extra')) {
      return;
    }

    normalizeRealmManagedRegisterFields(registerForm);

    const provinceOptions = [
      'AB', 'BC', 'MB', 'NB', 'NL', 'NS', 'NT', 'NU', 'ON', 'PE', 'QC', 'SK', 'YT'
    ];

    const firstNameInput = registerForm.querySelector('#firstName');
    const lastNameInput = registerForm.querySelector('#lastName');
    const firstNameGroup = firstNameInput?.closest('.form-group');
    const lastNameGroup = lastNameInput?.closest('.form-group');
    if (firstNameGroup) {
      firstNameGroup.style.display = 'none';
    }
    if (lastNameGroup) {
      lastNameGroup.style.display = 'none';
    }

    const fullNameGroup = document.createElement('div');
    fullNameGroup.className = 'form-group cbx-form-group cbx-register-extra';
    fullNameGroup.innerHTML = `
      <label for="cbx-fullName" class="pf-c-form__label pf-c-form__label-text">Full name *</label>
      <input id="cbx-fullName" name="user.attributes.fullName" class="pf-c-form-control" type="text" required placeholder="Jean-Francois O'Brien" autocomplete="name" minlength="2" maxlength="100" />
    `;

    const addressBlock = document.createElement('fieldset');
    addressBlock.className = 'cbx-register-fieldset cbx-register-extra';
    addressBlock.innerHTML = `
      <legend>Address</legend>
      <div class="form-group cbx-form-group">
        <label for="cbx-street" class="pf-c-form__label pf-c-form__label-text">Street *</label>
        <input id="cbx-street" name="user.attributes.street" class="pf-c-form-control" type="text" required placeholder="123 Maple St" autocomplete="street-address" minlength="3" />
      </div>

      <div class="cbx-grid-2">
        <div class="form-group cbx-form-group">
          <label for="cbx-city" class="pf-c-form__label pf-c-form__label-text">City *</label>
          <input id="cbx-city" name="user.attributes.city" class="pf-c-form-control" type="text" required placeholder="Toronto" autocomplete="address-level2" minlength="2" />
        </div>
        <div class="form-group cbx-form-group">
          <label for="cbx-postalCode" class="pf-c-form__label pf-c-form__label-text">Postal code *</label>
          <input id="cbx-postalCode" name="user.attributes.postalCode" class="pf-c-form-control" type="text" required placeholder="A1A 1A1" autocomplete="postal-code" pattern="^[A-Za-z]\d[A-Za-z][ -]?\d[A-Za-z]\d$" />
        </div>
      </div>

      <div class="cbx-grid-2">
        <div class="form-group cbx-form-group">
          <label for="cbx-province" class="pf-c-form__label pf-c-form__label-text">Province *</label>
          <select id="cbx-province" name="user.attributes.province" class="pf-c-form-control" required>
            <option value="" selected disabled>Select</option>
            ${provinceOptions.map((p) => `<option value="${p}">${p}</option>`).join('')}
          </select>
        </div>
        <div class="form-group cbx-form-group">
          <label for="cbx-country" class="pf-c-form__label pf-c-form__label-text">Country *</label>
          <input id="cbx-country" name="user.attributes.country" class="pf-c-form-control" type="text" value="Canada" required autocomplete="country-name" />
        </div>
      </div>
    `;

    const nasGroup = document.createElement('div');
    nasGroup.className = 'form-group cbx-form-group cbx-register-extra';
    nasGroup.innerHTML = `
      <label for="cbx-nas" class="pf-c-form__label pf-c-form__label-text">NAS *</label>
      <input id="cbx-nas" name="user.attributes.nas" class="pf-c-form-control" type="text" required placeholder="123 456 789" autocomplete="off" pattern="^\\d{3}[\\s-]?\\d{3}[\\s-]?\\d{3}$" />
    `;

    const usernameGroup = registerForm.querySelector('#username')?.closest('.form-group');
    const emailGroup = registerForm.querySelector('#email')?.closest('.form-group');
    const passwordGroup = registerForm.querySelector('#password')?.closest('.form-group');
    const confirmGroup = registerForm.querySelector('#password-confirm')?.closest('.form-group');
    const submitGroup = registerForm.querySelector('#kc-form-options')?.closest('.form-group');

    [usernameGroup, emailGroup, passwordGroup, confirmGroup].forEach((group) => {
      if (group) {
        group.classList.add('cbx-form-group');
      }
    });

    const orderedGroups = [
      fullNameGroup,
      usernameGroup,
      emailGroup,
      passwordGroup,
      confirmGroup,
      addressBlock,
      nasGroup
    ].filter(Boolean);

    const anchor = submitGroup || registerForm.lastElementChild;
    orderedGroups.forEach((node) => {
      if (anchor) {
        registerForm.insertBefore(node, anchor);
      } else {
        registerForm.appendChild(node);
      }
    });

    const passwordInput = registerForm.querySelector('#password');
    const confirmInput = registerForm.querySelector('#password-confirm');
    const emailInput = registerForm.querySelector('#email');
    const passBtn = registerForm.querySelector('button[aria-controls="password"]');
    const confirmBtn = registerForm.querySelector('button[aria-controls="password-confirm"]');
    const toggleButtons = [passBtn, confirmBtn].filter(Boolean);

    const updateToggleUi = (isVisible) => {
      toggleButtons.forEach((btn) => {
        btn.classList.add('cbx-password-toggle');
        btn.type = 'button';
        btn.setAttribute('aria-label', isVisible ? 'Hide password' : 'Show password');
        btn.innerHTML = isVisible ? EYE_OFF_ICON : EYE_ICON;
      });
    };

    const syncPasswordVisibility = (event) => {
      event.preventDefault();
      event.stopPropagation();
      if (!passwordInput || !confirmInput) {
        return;
      }
      const isVisible = passwordInput.type === 'text';
      const nextType = isVisible ? 'password' : 'text';
      passwordInput.type = nextType;
      confirmInput.type = nextType;
      updateToggleUi(nextType === 'text');
    };

    toggleButtons.forEach((btn) => {
      btn.removeAttribute('data-password-toggle');
      btn.addEventListener('click', syncPasswordVisibility, true);
    });
    updateToggleUi(false);
    enhancePasswordToggleIcons();

    const fullNameInput = registerForm.querySelector('#cbx-fullName');
    const postalInput = registerForm.querySelector('#cbx-postalCode');
    const nasInput = registerForm.querySelector('#cbx-nas');
    const streetInput = registerForm.querySelector('#cbx-street');
    const cityInput = registerForm.querySelector('#cbx-city');
    const provinceInput = registerForm.querySelector('#cbx-province');
    const countryInput = registerForm.querySelector('#cbx-country');

    fullNameInput?.addEventListener('input', () => {
      fullNameInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    emailInput?.addEventListener('input', () => {
      // Prevent stale custom validity from blocking submit before submit handler runs.
      emailInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    emailInput?.addEventListener('change', () => {
      emailInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    nasInput?.addEventListener('input', () => {
      // Prevent stale NAS custom validity from blocking submit before submit handler runs.
      nasInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    nasInput?.addEventListener('change', () => {
      nasInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    postalInput?.addEventListener('input', () => {
      // Prevent stale postal code custom validity from blocking submit.
      postalInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    postalInput?.addEventListener('change', () => {
      postalInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    passwordInput?.addEventListener('input', () => {
      // Prevent stale password custom validity from blocking submit.
      passwordInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    passwordInput?.addEventListener('change', () => {
      passwordInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    confirmInput?.addEventListener('input', () => {
      // Prevent stale password confirmation custom validity from blocking submit.
      confirmInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    confirmInput?.addEventListener('change', () => {
      confirmInput.setCustomValidity('');
      updateRegisterDraftFromForm(registerForm);
    });

    emailInput?.setAttribute('maxlength', '254');
    passwordInput?.setAttribute('minlength', '12');
    confirmInput?.setAttribute('minlength', '12');

    ['#username', '#email', '#cbx-street', '#cbx-city', '#cbx-province', '#cbx-postalCode', '#cbx-country', '#cbx-nas']
      .forEach((selector) => {
        const field = registerForm.querySelector(selector);
        if (!field) {
          return;
        }
        field.addEventListener('input', () => updateRegisterDraftFromForm(registerForm));
        field.addEventListener('change', () => updateRegisterDraftFromForm(registerForm));
      });

    restoreRegisterDraftToForm(registerForm);

    registerForm.addEventListener('submit', (event) => {
      updateRegisterDraftFromForm(registerForm);

      const fullName = normalizeSpaces(fullNameInput?.value || '');
      const fullNameDigitsOnly = /^\d+$/.test(fullName);
      if (!fullNameInput || fullName.length < 2 || fullName.length > 100 || !FULL_NAME_REGEX.test(fullName) || fullNameDigitsOnly) {
        fullNameInput?.setCustomValidity("Use 2-100 letters, spaces, hyphens, or apostrophes");
        fullNameInput?.reportValidity();
        event.preventDefault();
        return;
      }
      fullNameInput.value = fullName;
      fullNameInput.setCustomValidity('');

      const email = (emailInput?.value || '').trim().toLowerCase();
      if (!emailInput || email.length === 0 || email.length > 254 || !EMAIL_REGEX.test(email)) {
        emailInput?.setCustomValidity('Enter a valid email address');
        emailInput?.reportValidity();
        event.preventDefault();
        return;
      }
      emailInput.value = email;
      emailInput.setCustomValidity('');

      const normalizedStreet = normalizeSpaces(streetInput?.value || '');
      const normalizedCity = normalizeSpaces(cityInput?.value || '');
      const normalizedProvince = normalizeSpaces(provinceInput?.value || '').toUpperCase();
      const normalizedCountry = normalizeSpaces(countryInput?.value || '');
      if (!streetInput || !normalizedStreet || !cityInput || !normalizedCity || !provinceInput || !normalizedProvince || !countryInput || !normalizedCountry) {
        event.preventDefault();
        return;
      }
      streetInput.value = normalizedStreet;
      cityInput.value = normalizedCity;
      provinceInput.value = normalizedProvince;
      countryInput.value = normalizedCountry;

      const postalCompact = compactPostalCode(postalInput?.value || '');
      if (!postalInput || !POSTAL_REGEX.test(postalCompact)) {
        postalInput?.setCustomValidity('Use Canadian format A1A 1A1');
        postalInput?.reportValidity();
        event.preventDefault();
        return;
      }
      postalInput.value = `${postalCompact.slice(0, 3).toUpperCase()} ${postalCompact.slice(3).toUpperCase()}`;
      postalInput.setCustomValidity('');

      const nasDigits = compactNas(nasInput?.value || '');
      if (!nasInput || !/^\d{9}$/.test(nasDigits)) {
        nasInput?.setCustomValidity('NAS must contain exactly 9 digits');
        nasInput?.reportValidity();
        event.preventDefault();
        return;
      }
      nasInput.value = nasDigits;
      nasInput.setCustomValidity('');

      const password = passwordInput?.value || '';
      if (
        !passwordInput
        || password.length < 12
        || !/[A-Z]/.test(password)
        || !/[a-z]/.test(password)
        || !/\d/.test(password)
        || !PASSWORD_SPECIAL_REGEX.test(password)
        || password.toLowerCase() === email
      ) {
        passwordInput?.setCustomValidity('Password must be 12+ chars with uppercase, lowercase, number, special char, and not equal to email');
        passwordInput?.reportValidity();
        event.preventDefault();
        return;
      }
      passwordInput.setCustomValidity('');

      const confirm = confirmInput?.value || '';
      if (!confirmInput || confirm !== password) {
        confirmInput?.setCustomValidity('Password confirmation must match password');
        confirmInput?.reportValidity();
        event.preventDefault();
        return;
      }
      confirmInput.setCustomValidity('');

      // Keep Keycloak required first/last name fields in sync while the UI uses fullName only.
      const [firstName = '', ...rest] = fullName.split(' ');
      const lastName = rest.join(' ').trim() || firstName;
      if (firstNameInput && lastNameInput) {
        firstNameInput.value = firstName;
        lastNameInput.value = lastName;
      }
    });
  };

  const simplifyTotpSetupPage = () => {
    const isTotpSetup =
      window.location.pathname.includes('/login-actions/required-action') ||
      window.location.pathname.includes('/login-actions/authenticate');

    if (!isTotpSetup) {
      return;
    }

    // Remove optional device label field from TOTP setup page.
    const deviceNameInput = document.querySelector(
      'input[name="userLabel"], input#userLabel, input#totpLabel'
    );
    if (deviceNameInput) {
      const group = deviceNameInput.closest('.form-group') || deviceNameInput.parentElement;
      if (group) {
        group.remove();
      } else {
        deviceNameInput.remove();
      }
    }

    // Disable and hide "sign out from other devices" option.
    const signOutCheckbox = document.querySelector(
      'input[name="logout-sessions"], input[name="logoutSessions"], input#logout-sessions'
    );
    if (signOutCheckbox) {
      signOutCheckbox.checked = false;
      signOutCheckbox.value = 'false';

      const group = signOutCheckbox.closest('.form-group') || signOutCheckbox.parentElement;
      if (group) {
        group.remove();
      } else {
        signOutCheckbox.remove();
      }

      // Ensure form submits an explicit false value.
      const form = document.getElementById('kc-totp-settings-form') || signOutCheckbox.form;
      if (form && !form.querySelector('input[type="hidden"][name="logout-sessions"]')) {
        const hidden = document.createElement('input');
        hidden.type = 'hidden';
        hidden.name = 'logout-sessions';
        hidden.value = 'false';
        form.appendChild(hidden);
      }
    }
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      redirectIfAlreadyAuthenticatedForRegistration();
      applyAuthLayoutScaffold();
      addHomeButton();
      enhancePasswordToggleIcons();
      enhanceRegisterFields();
      simplifyTotpSetupPage();
    });
  } else {
    redirectIfAlreadyAuthenticatedForRegistration();
    applyAuthLayoutScaffold();
    addHomeButton();
    enhancePasswordToggleIcons();
    enhanceRegisterFields();
    simplifyTotpSetupPage();
  }
})();

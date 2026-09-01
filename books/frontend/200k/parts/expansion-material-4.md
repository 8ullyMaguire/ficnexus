# Expansion Material: Comprehensive Guides and Extended Walkthroughs

*Detailed guides, real-world examples, and hands-on exercises to deepen understanding.*

---

## Comprehensive Guide: Building a Theme System

### The Complete Theme System

A theme system is one of the most important parts of a web application. It ensures visual consistency, supports accessibility, and allows users to customize their experience. Let's build a complete theme system for FicHub.

### Step 1: Define Theme Tokens

Theme tokens are the building blocks of your design system. They're named values that represent design decisions:

```typescript
// lib/theme/tokens.ts
export interface ThemeTokens {
  colors: {
    bg: string;
    surface: string;
    surface2: string;
    surface3: string;
    text: string;
    muted: string;
    border: string;
    primary: string;
    primaryHover: string;
    success: string;
    warning: string;
    error: string;
    info: string;
  };
  typography: {
    fontFamily: string;
    monoFontFamily: string;
    sizes: Record<string, string>;
    weights: Record<string, number>;
    lineHeights: Record<string, number>;
  };
  spacing: Record<string, string>;
  radii: Record<string, string>;
  shadows: Record<string, string>;
  transitions: Record<string, string>;
}

export const darkTheme: ThemeTokens = {
  colors: {
    bg: '#0f1117',
    surface: '#171a23',
    surface2: '#1e2130',
    surface3: '#252a3a',
    text: '#f3f4f6',
    muted: '#9ca3af',
    border: '#2d3148',
    primary: '#6366f1',
    primaryHover: '#818cf8',
    success: '#10b981',
    warning: '#f59e0b',
    error: '#ef4444',
    info: '#3b82f6'
  },
  typography: {
    fontFamily: "'Inter', system-ui, -apple-system, sans-serif",
    monoFontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    sizes: {
      xs: '0.75rem',
      sm: '0.875rem',
      base: '1rem',
      lg: '1.125rem',
      xl: '1.25rem',
      '2xl': '1.5rem',
      '3xl': '1.875rem'
    },
    weights: {
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700
    },
    lineHeights: {
      tight: 1.25,
      normal: 1.5,
      relaxed: 1.75
    }
  },
  spacing: {
    xs: '0.25rem',
    sm: '0.5rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
    '2xl': '3rem'
  },
  radii: {
    sm: '4px',
    md: '8px',
    lg: '12px',
    xl: '16px',
    full: '9999px'
  },
  shadows: {
    sm: '0 1px 2px rgba(0, 0, 0, 0.2)',
    md: '0 4px 6px rgba(0, 0, 0, 0.3)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.4)',
    xl: '0 20px 25px rgba(0, 0, 0, 0.5)'
  },
  transitions: {
    fast: '150ms ease',
    normal: '200ms ease',
    slow: '300ms ease'
  }
};

export const lightTheme: ThemeTokens = {
  colors: {
    bg: '#f9fafb',
    surface: '#ffffff',
    surface2: '#f3f4f6',
    surface3: '#e5e7eb',
    text: '#111827',
    muted: '#6b7280',
    border: '#e5e7eb',
    primary: '#4f46e5',
    primaryHover: '#4338ca',
    success: '#059669',
    warning: '#d97706',
    error: '#dc2626',
    info: '#2563eb'
  },
  typography: darkTheme.typography,
  spacing: darkTheme.spacing,
  radii: darkTheme.radii,
  shadows: {
    sm: '0 1px 2px rgba(0, 0, 0, 0.05)',
    md: '0 4px 6px rgba(0, 0, 0, 0.1)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.1)',
    xl: '0 20px 25px rgba(0, 0, 0, 0.15)'
  },
  transitions: darkTheme.transitions
};
```

### Step 2: Theme Provider

```typescript
// lib/theme/provider.ts
import { darkTheme, lightTheme, type ThemeTokens } from './tokens';

type ThemeName = 'dark' | 'light' | 'system';

let currentTheme = $state<ThemeName>('system');
let resolvedTheme = $state<ThemeTokens>(darkTheme);

function resolveTheme(name: ThemeName): ThemeTokens {
  if (name === 'system') {
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
        ? darkTheme
        : lightTheme;
    }
    return darkTheme;
  }
  return name === 'dark' ? darkTheme : lightTheme;
}

function applyTheme(theme: ThemeTokens) {
  const root = document.documentElement;
  const entries = Object.entries(theme.colors) as [string, string][];
  for (const [key, value] of entries) {
    root.style.setProperty(`--color-${key}`, value);
  }
}

export const themeProvider = {
  get current() { return currentTheme; },
  get resolved() { return resolvedTheme; },
  
  set theme(name: ThemeName) {
    currentTheme = name;
    resolvedTheme = resolveTheme(name);
    applyTheme(resolvedTheme);
    localStorage.setItem('theme', name);
  },
  
  toggle() {
    this.theme = currentTheme === 'dark' ? 'light' : 'dark';
  },
  
  init() {
    // Load saved preference
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem('theme') as ThemeName;
      if (saved) currentTheme = saved;
    }
    
    resolvedTheme = resolveTheme(currentTheme);
    
    if (typeof document !== 'undefined') {
      applyTheme(resolvedTheme);
      
      // Listen for system theme changes
      window.matchMedia('(prefers-color-scheme: dark)')
        .addEventListener('change', () => {
          if (currentTheme === 'system') {
            resolvedTheme = resolveTheme('system');
            applyTheme(resolvedTheme);
          }
        });
    }
  }
};
```

### Step 3: Theme Toggle Component

```svelte
<!-- ThemeToggle.svelte -->
<script lang="ts">
  import { themeProvider } from '$lib/theme/provider';
  
  let current = $derived(themeProvider.current);
  
  function toggle() {
    themeProvider.toggle();
  }
</script>

<button 
  class="theme-toggle" 
  onclick={toggle}
  aria-label="Toggle theme (currently {current})"
>
  {#if current === 'dark'}
    <span class="icon">☀️</span>
    <span class="label">Light</span>
  {:else}
    <span class="icon">🌙</span>
    <span class="label">Dark</span>
  {/if}
</button>

<style>
  .theme-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.9rem;
    transition: all var(--transition-fast);
  }
  
  .theme-toggle:hover {
    background: var(--color-surface-3);
    border-color: var(--color-primary);
  }
  
  .icon {
    font-size: 1.1rem;
  }
</style>
```

### Step 4: Usage in Layout

```svelte
<!-- +layout.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { themeProvider } from '$lib/theme/provider';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  
  onMount(() => {
    themeProvider.init();
  });
</script>

<div class="app" data-theme={themeProvider.current}>
  <header>
    <nav>
      <a href="/" class="logo">FicHub</a>
      <ThemeToggle />
    </nav>
  </header>
  
  <main>
    <slot />
  </main>
  
  <footer>
    <p>Built with Svelte 5</p>
  </footer>
</div>
```

---

## Comprehensive Guide: Form Validation

### Building a Complete Form Validation System

Forms are everywhere, and validation is critical. Let's build a comprehensive validation system.

### Step 1: Define Validators

```typescript
// lib/validation/validators.ts
export type Validator<T> = (value: T) => string | null;

export const required: Validator<any> = (value) => {
  if (value === null || value === undefined || value === '') {
    return 'This field is required';
  }
  return null;
};

export const email: Validator<string> = (value) => {
  if (!value) return null;
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!regex.test(value)) {
    return 'Please enter a valid email address';
  }
  return null;
};

export const minLength = (min: number): Validator<string> => (value) => {
  if (!value) return null;
  if (value.length < min) {
    return `Must be at least ${min} characters`;
  }
  return null;
};

export const maxLength = (max: number): Validator<string> => (value) => {
  if (!value) return null;
  if (value.length > max) {
    return `Must be no more than ${max} characters`;
  }
  return null;
};

export const pattern = (regex: RegExp, message: string): Validator<string> => (value) => {
  if (!value) return null;
  if (!regex.test(value)) {
    return message;
  }
  return null;
};

export const min = (min: number): Validator<number> => (value) => {
  if (value === null || value === undefined) return null;
  if (value < min) {
    return `Must be at least ${min}`;
  }
  return null;
};

export const max = (max: number): Validator<number> => (value) => {
  if (value === null || value === undefined) return null;
  if (value > max) {
    return `Must be no more than ${max}`;
  }
  return null;
};

export const url: Validator<string> = (value) => {
  if (!value) return null;
  try {
    new URL(value);
    return null;
  } catch {
    return 'Please enter a valid URL';
  }
};

export const matches = (otherValue: () => any, message: string): Validator<any> => (value) => {
  if (value !== otherValue()) {
    return message;
  }
  return null;
};

export const compose = <T>(...validators: Validator<T>[]): Validator<T> => (value) => {
  for (const validator of validators) {
    const error = validator(value);
    if (error) return error;
  }
  return null;
};
```

### Step 2: Form State Manager

```typescript
// lib/validation/formState.ts
type ValidatorMap<T> = { [K in keyof T]?: Validator<T[K]> };

interface FieldState<V> {
  value: V;
  error: string | null;
  touched: boolean;
  dirty: boolean;
}

interface FormState<T> {
  fields: { [K in keyof T]: FieldState<T[K]> };
  isValid: boolean;
  isDirty: boolean;
  isSubmitting: boolean;
  submitError: string | null;
}

export function createFormState<T extends Record<string, any>>(
  initialValues: T,
  validators?: ValidatorMap<T>
) {
  const fields = {} as any;
  for (const key of Object.keys(initialValues)) {
    fields[key] = {
      value: initialValues[key],
      error: null,
      touched: false,
      dirty: false
    };
  }

  let form = $state<FormState<T>>({
    fields,
    isValid: true,
    isDirty: false,
    isSubmitting: false,
    submitError: null
  });

  function validateField(key: keyof T): string | null {
    const validator = validators?.[key];
    if (!validator) return null;
    return validator(form.fields[key].value);
  }

  function setValue<K extends keyof T>(key: K, value: T[K]) {
    form.fields[key].value = value;
    form.fields[key].dirty = value !== initialValues[key];
    if (form.fields[key].touched) {
      form.fields[key].error = validateField(key);
    }
    updateFormState();
  }

  function setTouched<K extends keyof T>(key: K) {
    form.fields[key].touched = true;
    form.fields[key].error = validateField(key);
    updateFormState();
  }

  function touchAll() {
    for (const key of Object.keys(form.fields) as (keyof T)[]) {
      form.fields[key].touched = true;
      form.fields[key].error = validateField(key);
    }
    updateFormState();
  }

  function updateFormState() {
    const errors = Object.values(form.fields)
      .map((f: any) => f.error)
      .filter(Boolean);
    form.isValid = errors.length === 0;
    form.isDirty = Object.values(form.fields)
      .some((f: any) => f.dirty);
  }

  function reset() {
    for (const key of Object.keys(initialValues) as (keyof T)[]) {
      form.fields[key] = {
        value: initialValues[key],
        error: null,
        touched: false,
        dirty: false
      };
    }
    form.isSubmitting = false;
    form.submitError = null;
    updateFormState();
  }

  function getValues(): T {
    const values = {} as any;
    for (const key of Object.keys(form.fields) as (keyof T)[]) {
      values[key] = form.fields[key].value;
    }
    return values;
  }

  return {
    get state() { return form; },
    setValue,
    setTouched,
    touchAll,
    reset,
    getValues,
    validate() {
      touchAll();
      return form.isValid;
    }
  };
}
```

### Step 3: Form Field Component

```svelte
<!-- FormField.svelte -->
<script lang="ts">
  interface Props {
    label: string;
    error?: string | null;
    hint?: string;
    required?: boolean;
    for?: string;
  }

  let { label, error, hint, required = false, for: forId }: Props = $props();
</script>

<div class="form-field" class:error={!!error}>
  <label for={forId}>
    {label}
    {#if required}
      <span class="required" aria-hidden="true">*</span>
    {/if}
  </label>
  
  <div class="field-content">
    <slot />
  </div>
  
  {#if error}
    <p class="error-text" role="alert">{error}</p>
  {:else if hint}
    <p class="hint-text">{hint}</p>
  {/if}
</div>

<style>
  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  
  label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text);
  }
  
  .required {
    color: var(--color-error);
  }
  
  .field-content {
    display: flex;
    flex-direction: column;
  }
  
  .error-text {
    color: var(--color-error);
    font-size: 0.8rem;
    margin: 0;
  }
  
  .hint-text {
    color: var(--color-muted);
    font-size: 0.8rem;
    margin: 0;
  }
</style>
```

### Step 4: Complete Form Example

```svelte
<!-- RegistrationForm.svelte -->
<script lang="ts">
  import { createFormState } from '$lib/validation/formState';
  import { required, email, minLength, compose, matches } from '$lib/validation/validators';
  import FormField from './FormField.svelte';

  let { onSuccess } = $props();

  const form = createFormState(
    { username: '', email: '', password: '', confirmPassword: '' },
    {
      username: compose(required, minLength(3)),
      email: compose(required, email),
      password: compose(required, minLength(8)),
      confirmPassword: compose(
        required,
        matches(() => form.state.fields.password.value, 'Passwords do not match')
      )
    }
  );

  async function handleSubmit() {
    if (!form.validate()) return;

    form.state.isSubmitting = true;
    form.state.submitError = null;

    try {
      const values = form.getValues();
      // Submit to API
      console.log('Submitting:', values);
      form.reset();
      onSuccess?.();
    } catch (e) {
      form.state.submitError = 'Registration failed. Please try again.';
    } finally {
      form.state.isSubmitting = false;
    }
  }
</script>

<form onsubmit|preventDefault={handleSubmit} class="registration-form">
  <FormField 
    label="Username" 
    error={form.state.fields.username.error}
    required
    for="username"
  >
    <input
      id="username"
      type="text"
      value={form.state.fields.username.value}
      oninput={(e) => form.setValue('username', e.target.value)}
      onblur={() => form.setTouched('username')}
      placeholder="Choose a username"
    />
  </FormField>

  <FormField 
    label="Email" 
    error={form.state.fields.email.error}
    required
    for="email"
  >
    <input
      id="email"
      type="email"
      value={form.state.fields.email.value}
      oninput={(e) => form.setValue('email', e.target.value)}
      onblur={() => form.setTouched('email')}
      placeholder="your@email.com"
    />
  </FormField>

  <FormField 
    label="Password" 
    error={form.state.fields.password.error}
    hint="Must be at least 8 characters"
    required
    for="password"
  >
    <input
      id="password"
      type="password"
      value={form.state.fields.password.value}
      oninput={(e) => form.setValue('password', e.target.value)}
      onblur={() => form.setTouched('password')}
      placeholder="Create a password"
    />
  </FormField>

  <FormField 
    label="Confirm Password" 
    error={form.state.fields.confirmPassword.error}
    required
    for="confirm-password"
  >
    <input
      id="confirm-password"
      type="password"
      value={form.state.fields.confirmPassword.value}
      oninput={(e) => form.setValue('confirmPassword', e.target.value)}
      onblur={() => form.setTouched('confirmPassword')}
      placeholder="Confirm your password"
    />
  </FormField>

  {#if form.state.submitError}
    <p class="submit-error" role="alert">{form.state.submitError}</p>
  {/if}

  <button 
    type="submit" 
    disabled={!form.state.isValid || form.state.isSubmitting}
    class="submit-btn"
  >
    {form.state.isSubmitting ? 'Creating Account...' : 'Create Account'}
  </button>
</form>

<style>
  .registration-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    max-width: 400px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  input {
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.95rem;
    transition: border-color 0.2s ease;
  }
  
  input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
  }
  
  .error input {
    border-color: var(--color-error);
  }
  
  .submit-error {
    color: var(--color-error);
    text-align: center;
    padding: 0.75rem;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius);
  }
  
  .submit-btn {
    padding: 0.75rem 1.5rem;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s ease;
  }
  
  .submit-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }
  
  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

---

## Comprehensive Guide: Error Handling Patterns

### Error Boundaries

Svelte doesn't have built-in error boundaries like React, but you can implement them:

```svelte
<!-- ErrorBoundary.svelte -->
<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  
  let { children, fallback } = $props();
  let error = $state(null);
  let errorInfo = $state('');

  function handleError(event: ErrorEvent) {
    error = event.error;
    errorInfo = event.message;
    event.preventDefault();
  }

  function handleUnhandledRejection(event: PromiseRejectionEvent) {
    error = event.reason;
    errorInfo = String(event.reason);
    event.preventDefault();
  }

  onMount(() => {
    window.addEventListener('error', handleError);
    window.addEventListener('unhandledrejection', handleUnhandledRejection);
  });

  onDestroy(() => {
    window.removeEventListener('error', handleError);
    window.removeEventListener('unhandledrejection', handleUnhandledRejection);
  });
</script>

{#if error}
  {#if fallback}
    {@render fallback(error, errorInfo)}
  {:else}
    <div class="error-boundary">
      <h2>Something went wrong</h2>
      <p>{errorInfo}</p>
      <button onclick={() => { error = null; errorInfo = ''; }}>
        Try Again
      </button>
    </div>
  {/if}
{:else}
  {@render children()}
{/if}

<style>
  .error-boundary {
    text-align: center;
    padding: 3rem;
    max-width: 500px;
    margin: 0 auto;
  }
  
  .error-boundary h2 {
    color: var(--color-error);
    margin-bottom: 1rem;
  }
  
  .error-boundary p {
    color: var(--color-muted);
    margin-bottom: 1.5rem;
  }
  
  .error-boundary button {
    padding: 0.75rem 1.5rem;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
  }
</style>
```

### Error Handling in API Calls

```typescript
// lib/api/errorHandler.ts
import type { ApiError } from './client';

export interface ErrorHandlerOptions {
  onNetworkError?: () => void;
  onAuthError?: () => void;
  onNotFound?: () => void;
  onServerError?: (status: number) => void;
  onUnknownError?: (error: unknown) => void;
}

export function handleApiError(
  error: unknown, 
  options: ErrorHandlerOptions = {}
): string {
  if (error instanceof ApiError) {
    switch (error.status) {
      case 401:
      case 403:
        options.onAuthError?.();
        return 'Authentication required. Please log in.';
      
      case 404:
        options.onNotFound?.();
        return 'Resource not found.';
      
      case 429:
        return 'Too many requests. Please wait and try again.';
      
      case 500:
      case 502:
      case 503:
        options.onServerError?.(error.status);
        return 'Server error. Please try again later.';
      
      default:
        return `Error ${error.status}: ${error.message}`;
    }
  }

  if (error instanceof TypeError && error.message.includes('fetch')) {
    options.onNetworkError?.();
    return 'Network error. Please check your connection.';
  }

  options.onUnknownError?.(error);
  return 'An unexpected error occurred.';
}
```

### Toast Error Display

```svelte
<!-- ErrorToast.svelte -->
<script lang="ts">
  import { fly } from 'svelte/transition';
  
  let { message, type = 'error', onDismiss, duration = 5000 } = $props();
  
  let visible = $state(true);
  
  $effect(() => {
    if (duration > 0) {
      const timer = setTimeout(() => {
        visible = false;
        onDismiss?.();
      }, duration);
      return () => clearTimeout(timer);
    }
  });
</script>

{#if visible}
  <div 
    class="toast toast-{type}"
    transition:fly={{ y: -20, duration: 200 }}
    role="alert"
  >
    <span class="toast-icon">
      {#if type === 'error'}⚠️
      {:else if type === 'success'}✓
      {:else if type === 'warning'}⚡
      {:else}ℹ️
      {/if}
    </span>
    <span class="toast-message">{message}</span>
    <button class="toast-close" onclick={() => { visible = false; onDismiss?.(); }}>
      ×
    </button>
  </div>
{/if}

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    border-radius: var(--radius);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-lg);
    min-width: 300px;
    max-width: 500px;
  }
  
  .toast-error { border-color: var(--color-error); }
  .toast-success { border-color: var(--color-success); }
  .toast-warning { border-color: var(--color-warning); }
  .toast-info { border-color: var(--color-info); }
  
  .toast-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }
  
  .toast-message {
    flex: 1;
    font-size: 0.9rem;
  }
  
  .toast-close {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0;
    flex-shrink: 0;
  }
  
  .toast-close:hover {
    color: var(--color-text);
  }
</style>
```

---

## Extended Practice Projects

### Project 1: Build a Complete Blog System

**Features:**
- Create, edit, delete posts
- Rich text editor
- Image uploads
- Categories and tags
- Search
- Comments
- RSS feed
- SEO meta tags

**Technical Requirements:**
- SvelteKit with server-side rendering
- Markdown parsing
- Image optimization
- Database integration
- API design
- Authentication

### Project 2: Build a Real-Time Chat Application

**Features:**
- Private and group chats
- Message history
- Typing indicators
- Read receipts
- File sharing
- Emoji support
- Message search

**Technical Requirements:**
- WebSocket connection
- State synchronization
- Offline support
- Push notifications
- Media handling

### Project 3: Build a Dashboard Application

**Features:**
- Multiple widget types (charts, tables, metrics)
- Drag-and-drop layout
- Real-time data updates
- Filtering and drill-down
- Export to PDF/CSV
- Responsive design

**Technical Requirements:**
- Chart.js or D3.js
- WebSocket for real-time data
- Virtual scrolling for large tables
- Print CSS
- Keyboard navigation

### Project 4: Build an E-Commerce Frontend

**Features:**
- Product listing with filters
- Shopping cart
- Checkout flow
- Order history
- User accounts
- Wishlist
- Product reviews

**Technical Requirements:**
- Complex state management
- Form validation
- Payment integration (Stripe)
- Email notifications
- SEO optimization

### Project 5: Build a Social Media Dashboard

**Features:**
- Feed with infinite scroll
- Post creation with rich media
- Like, comment, share
- User profiles
- Direct messaging
- Notifications
- Search and explore

**Technical Requirements:**
- Real-time updates
- Image/video handling
- Virtual scrolling
- Offline support
- Push notifications

---

## Complete CSS Animation Library

### Loading Animations

```css
/* Spinner */
@keyframes spin {
  to { transform: rotate(360deg); }
}
.spinner {
  width: 2rem;
  height: 2rem;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

/* Dots */
@keyframes dots {
  0%, 20% { content: '.'; }
  40% { content: '..'; }
  60%, 100% { content: '...'; }
}
.loading-dots::after {
  content: '.';
  animation: dots 1.5s steps(5, end) infinite;
}

/* Pulse */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
.pulse {
  animation: pulse 2s ease-in-out infinite;
}

/* Skeleton */
@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}
.skeleton {
  background: linear-gradient(90deg, 
    var(--color-surface) 25%, 
    var(--color-surface-2) 50%, 
    var(--color-surface) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--radius);
}
```

### Transition Animations

```css
/* Fade */
.fade-enter { opacity: 0; }
.fade-enter-active { opacity: 1; transition: opacity 0.3s ease; }
.fade-exit { opacity: 1; }
.fade-exit-active { opacity: 0; transition: opacity 0.3s ease; }

/* Slide */
.slide-enter { transform: translateY(20px); opacity: 0; }
.slide-enter-active { transform: translateY(0); opacity: 1; transition: all 0.3s ease; }
.slide-exit { transform: translateY(0); opacity: 1; }
.slide-exit-active { transform: translateY(-20px); opacity: 0; transition: all 0.3s ease; }

/* Scale */
.scale-enter { transform: scale(0.9); opacity: 0; }
.scale-enter-active { transform: scale(1); opacity: 1; transition: all 0.2s ease; }
.scale-exit { transform: scale(1); opacity: 1; }
.scale-exit-active { transform: scale(0.9); opacity: 0; transition: all 0.2s ease; }
```

### Micro-Interaction Animations

```css
/* Button hover */
.btn {
  transition: all 0.2s ease;
}
.btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.btn:active {
  transform: translateY(0);
}

/* Card hover */
.card {
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}
.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

/* Focus ring */
.focus-ring:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  transition: outline-offset 0.1s ease;
}

/* Checkbox */
.checkbox {
  transition: transform 0.15s ease;
}
.checkbox:checked {
  transform: scale(1.1);
}
```

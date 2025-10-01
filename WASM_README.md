# Chronos WebAssembly Frontend

This WebAssembly frontend provides a complete user interface for all Chronos authentication features.

## Features

The WebAssembly frontend includes:

- **User Authentication**
  - Login with email/password
  - User registration
  - Forgot password functionality
  - Password reset (with token)
  - Secure logout (single device or all devices)

- **Profile Management**
  - View user profile
  - Update profile information
  - Change password

- **Security Features**
  - JWT token management
  - Automatic token refresh
  - Local storage for session persistence
  - Rate limiting compliance
  - CORS support

- **Modern UI/UX**
  - Responsive design
  - Tab-based navigation
  - Real-time form validation
  - Loading states and error handling
  - Smooth animations and transitions

## Prerequisites

- Rust toolchain (latest stable)
- `wasm-pack` for WebAssembly compilation
- A web server for serving static files
- Chronos backend server running

## Building

1. **Install wasm-pack** (if not already installed):
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. **Build the WebAssembly package**:
   ```bash
   ./build-wasm.sh
   ```

   Or manually:
   ```bash
   wasm-pack build --target web --out-dir pkg --release
   ```

## Running

1. **Start the Chronos backend**:
   ```bash
   cargo run
   ```
   The backend will run on `http://localhost:3000` by default.

2. **Serve the frontend** using any static file server:

   Using Python:
   ```bash
   python3 -m http.server 8080
   ```

   Using Node.js:
   ```bash
   npx serve .
   ```

   Using Rust:
   ```bash
   cargo install basic-http-server
   basic-http-server .
   ```

3. **Open your browser** and navigate to the frontend URL (e.g., `http://localhost:8080`)

## Project Structure

```
chronos/
├── src/
│   ├── lib.rs              # WebAssembly bindings and exports
│   ├── wasm_frontend/
│   │   ├── mod.rs          # Frontend initialization and event handling
│   │   └── ui.rs           # UI utilities and components
│   └── ...                 # Backend code
├── index.html              # Main HTML file
├── styles.css              # CSS styles
├── build-wasm.sh          # Build script
└── pkg/                   # Generated WebAssembly package (after build)
```

## API Integration

The frontend communicates with the Chronos backend via HTTP requests to these endpoints:

- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `GET /api/auth/profile` - Get user profile
- `PUT /api/auth/profile` - Update user profile
- `POST /api/auth/change-password` - Change password
- `POST /api/auth/forgot-password` - Request password reset
- `POST /api/auth/reset-password` - Reset password with token
- `POST /api/auth/refresh` - Refresh JWT token

## Configuration

The frontend can be configured by modifying the `ChronosAuth` constructor call in `src/lib.rs`:

```rust
// Default: http://localhost:3000
let auth = ChronosAuth::new(None);

// Custom backend URL
let auth = ChronosAuth::new(Some("https://your-api.com".to_string()));
```

## Browser Compatibility

The WebAssembly frontend supports:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Security Considerations

- JWT tokens are stored in `localStorage`
- Automatic token refresh prevents session expiration
- CORS headers must be properly configured on the backend
- All API calls include proper authentication headers
- Form validation prevents common input attacks

## Development

For development with hot reloading:

1. Use `cargo watch` to rebuild on changes:
   ```bash
   cargo install cargo-watch
   cargo watch -x "build --lib"
   wasm-pack build --target web --out-dir pkg --dev
   ```

2. Use a development server with live reload:
   ```bash
   npx live-server .
   ```

## Troubleshooting

**WebAssembly module fails to load**:
- Ensure the backend server is running
- Check browser console for CORS errors
- Verify the WebAssembly build completed successfully

**Authentication fails**:
- Check network requests in browser DevTools
- Verify backend API endpoints are accessible
- Check for proper CORS configuration

**UI doesn't display correctly**:
- Ensure all CSS files are loading
- Check for JavaScript errors in console
- Verify WebAssembly initialization completed

## Performance

The WebAssembly frontend is optimized for:
- Fast initial load times
- Minimal bundle size
- Efficient DOM manipulation
- Smooth animations and transitions

## Contributing

To contribute to the WebAssembly frontend:

1. Make changes to Rust code in `src/lib.rs` or `src/wasm_frontend/`
2. Update HTML/CSS as needed
3. Build and test the changes
4. Submit a pull request

## License

This project is licensed under the same terms as the main Chronos project.
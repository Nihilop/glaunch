# GLaunch

GLaunch is a modern, cross-platform game launcher built with Tauri and Vue.js. It provides a unified interface to manage and launch games from different platforms like Steam, Epic Games, and Battle.net. With its sleek design and keyboard-first navigation, GLaunch offers a seamless gaming experience.

https://github.com/user-attachments/assets/1b8e9f65-aeff-4073-b212-998fac58fc7f





## 🎮 Features

### Platform Integration
- Steam library integration with automatic game detection
- Battle.net games support with authentication
- Epic Games Store integration
- Custom games folder support
- Automatic metadata fetching from IGDB
- Real-time game session tracking

### User Experience
- Keyboard-first navigation system
- Custom overlay system for in-game information
- Multiple view modes (grid/list)
- Game search and filtering
- Genre and tag-based organization
- Minimalist design with dark mode

### Technical Features
- Automatic updates with built-in updater
- Tray icon support with minimize functionality
- Windows startup integration
- Customizable settings
- Database import/export functionality
- System tray integration
- Games statistics tracking
- Media caching system

## 🚀 Development Setup

### Prerequisites
- Node.js (v18 or higher)
- Rust (latest stable)
- pnpm package manager
- Visual Studio Build Tools (Windows)

### Environment Variables
Create a `.env` file in the root directory with:
```env
STEAM_API_KEY=your_steam_api_key
EPIC_CLIENT_ID=your_epic_client_id
EPIC_CLIENT_ID_SECRET=your_epic_client_id_secret
BATTLENET_CLIENT_ID=your_battlenet_client_id
BATTLENET_CLIENT_SECRET=your_battlenet_client_secret
IGDB_CLIENT_ID=your_igdb_client_id
IGDB_CLIENT_SECRET=your_igdb_client_secret
```

### Installation
1. Clone the repository:
```bash
git clone https://github.com/yourusername/glaunch.git
cd glaunch
```

2. Install dependencies:
```bash
pnpm install
```

3. Run the development server:
```bash
pnpm tauri dev
```

### Build
To create a production build:
```bash
pnpm tauri build
```

## 📝 Contributing
Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) before submitting a pull request.

## 📜 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

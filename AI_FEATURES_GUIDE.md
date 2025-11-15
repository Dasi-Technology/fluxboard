# AI Features Guide

This guide explains how to use the AI-powered card description generation feature in Fluxboard.

## Overview

Fluxboard now includes AI-powered description generation using Google's Gemini API. Users can automatically generate card descriptions in two formats:
- **Bullet Points**: A concise list of 3-5 key points
- **Long Description**: A detailed 2-3 paragraph description with markdown formatting

## Setup

### 1. Get a Gemini API Key

1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Sign in with your Google account
3. Click "Create API Key"
4. Copy the generated API key

### 2. Configure the Backend

1. Navigate to the `backend` directory
2. Copy `.env.example` to `.env` if you haven't already:
   ```bash
   cp .env.example .env
   ```
3. Add your Gemini API key to the `.env` file:
   ```
   GEMINI_API_KEY=your_api_key_here
   ```

### 3. Restart the Backend

Restart the backend server to load the new configuration:
```bash
cd backend
cargo run
```

## How to Use

### Generating Descriptions

1. **Create or Edit a Card**: Open the card edit dialog by clicking on a card
2. **Enter a Title**: Type a descriptive title for your card (required)
3. **Add Context (Optional)**: If you have existing text in the description field, it will be used as context for the AI
4. **Choose a Format**:
   - Click **"AI Bullets"** to generate a bullet-point list
   - Click **"AI Long"** to generate a detailed description
5. **Review and Edit**: The generated description will appear in the description field. You can edit it as needed
6. **Save**: Click "Save Changes" to save the card

### Tips for Best Results

- **Use descriptive titles**: The AI uses the card title as the primary context
- **Provide context**: If you have existing notes or requirements, add them to the description field before generating
- **Iterate**: You can generate multiple times with different formats or contexts
- **Edit freely**: The generated text is a starting point - feel free to modify it

## Examples

### Example 1: Feature Card

**Title**: "User Authentication"
**Context**: "JWT-based authentication with refresh tokens"
**Generated Bullets**:
```
- Implement JWT token generation and validation
- Create secure login and registration endpoints
- Add refresh token mechanism for session management
- Integrate password hashing with bcrypt
- Handle token expiration and renewal
```

### Example 2: Bug Fix Card

**Title**: "Fix responsive layout on mobile"
**Context**: "Navigation menu breaks on screens smaller than 768px"
**Generated Long Description**:
```
The mobile responsive layout requires attention to ensure proper functionality across all 
device sizes. The primary issue affects the navigation menu, which currently breaks on 
screens smaller than 768px, causing menu items to overlap and become inaccessible.

To resolve this, we need to implement a mobile-first approach with appropriate media 
queries. This includes creating a hamburger menu for smaller screens, ensuring touch 
targets are appropriately sized, and testing across multiple device viewports. The 
solution should maintain accessibility standards while providing a smooth user experience.
```

## Technical Details

### Backend Architecture

- **AI Service** (`backend/src/services/ai_service.rs`): Handles communication with Gemini API
- **Card Handlers** (`backend/src/handlers/card_handlers.rs`): Provides the `/api/cards/ai/generate-description` endpoint
- **Configuration** (`backend/src/config.rs`): Manages the API key from environment variables

### Frontend Integration

- **API Client** (`frontend/src/lib/api.ts`): `generateDescription()` function
- **Edit Card Dialog** (`frontend/src/components/dialogs/edit-card-dialog.tsx`): UI buttons and state management

### API Endpoint

**POST** `/api/cards/ai/generate-description`

**Request Body**:
```json
{
  "title": "Card title",
  "context": "Optional context or existing description",
  "format": "bullets" | "long"
}
```

**Response**:
```json
{
  "description": "Generated markdown description"
}
```

## Troubleshooting

### "AI service not configured" Error

**Problem**: The backend returns an error saying the AI service is not configured.

**Solution**: 
- Ensure `GEMINI_API_KEY` is set in your `backend/.env` file
- Restart the backend server after adding the key

### Generation Button Disabled

**Problem**: The AI generation buttons are grayed out.

**Causes**:
- No card title entered (title is required)
- Currently generating (wait for completion)
- Card is being saved

**Solution**: Enter a card title before clicking the AI generation buttons

### Slow Generation

**Problem**: AI generation takes a long time.

**Note**: Generation typically takes 2-5 seconds depending on network latency and API response time. This is normal for AI-powered features.

### API Rate Limits

**Problem**: Gemini API returns rate limit errors.

**Solution**: 
- Wait a few moments before trying again
- Consider upgrading your Gemini API plan for higher limits
- Monitor your usage at [Google AI Studio](https://aistudio.google.com/)

## Privacy & Security

- Card titles and descriptions are sent to Google's Gemini API for processing
- The API key is stored securely in the backend environment variables
- No data is permanently stored by the AI service
- Review Google's [Gemini API Terms of Service](https://ai.google.dev/gemini-api/terms) for details

## Future Enhancements

Potential future improvements:
- Template-based generation for common card types
- Multi-language support
- Custom prompts and styles
- Batch generation for multiple cards
- Integration with other AI providers
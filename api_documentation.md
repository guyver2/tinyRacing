# Racing Simulation RESTful API

This document outlines the RESTful API endpoints for controlling the race simulation.

## Base URL

```
https://api.racing-sim.com/v1
```

## Authentication

All API requests require authentication using a bearer token:

```
Authorization: Bearer <your_token>
```

## Race Control Endpoints

### Get Race Status

**Endpoint:** `GET /race`

**Description:** Get the current race state and status.

**Response:**
```json
{
  "status": "success",
  "data": {
    "run_state": "running|paused|finished",
    "elapsed_time": 1234.56,
    "lap_count": 42,
    "cars": [
      {
        "number": 44,
        "position": 1,
        "lap": 42,
        "status": "racing",
        "driving_style": "normal",
        "tires": {
          "type": "soft",
          "condition": 78.5
        },
        "fuel": 42.5
      }
    ]
  }
}
```

### Start Race

**Endpoint:** `POST /race/start`

**Description:** Start or resume the race.

**Response:**
```json
{
  "status": "success",
  "message": "Race started!"
}
```

### Pause Race

**Endpoint:** `POST /race/pause`

**Description:** Pause the race.

**Response:**
```json
{
  "status": "success",
  "message": "Race paused."
}
```

### Stop Race

**Endpoint:** `POST /race/stop`

**Description:** Stop/finish the race.

**Response:**
```json
{
  "status": "success",
  "message": "Race stopped/finished manually."
}
```

## Car Control Endpoints

### Get Car Status

**Endpoint:** `GET /cars/{car_number}`

**Description:** Get the status of a specific car.

**Path Parameters:**
- `car_number` (integer): The car's racing number

**Response:**
```json
{
  "status": "success",
  "data": {
    "number": 44,
    "position": 1,
    "lap": 42,
    "status": "racing",
    "driving_style": "normal",
    "tires": {
      "type": "soft",
      "condition": 78.5
    },
    "fuel": 42.5
  }
}
```

### Set Driving Style

**Endpoint:** `PUT /cars/{car_number}/driving-style`

**Description:** Set the driving style for a specific car.

**Path Parameters:**
- `car_number` (integer): The car's racing number

**Request Body:**
```json
{
  "style": "relax|normal|aggressive"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Car 44 driving style set to Normal."
}
```

### Set Car Status

**Endpoint:** `PUT /cars/{car_number}/status`

**Description:** Set a car's status (e.g., DNF).

**Path Parameters:**
- `car_number` (integer): The car's racing number

**Request Body:**
```json
{
  "status": "dnf"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Car 44 set to DNF."
}
```

## Pit Stop Endpoints

### Request Pit Stop

**Endpoint:** `POST /cars/{car_number}/pit`

**Description:** Order a car to pit with optional tire change and refueling.

**Path Parameters:**
- `car_number` (integer): The car's racing number

**Request Body:**
```json
{
  "tires": "soft|medium|hard|intermediate|wet",  // Optional
  "refuel": 50.0  // Optional, percentage 0-100
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Car 44 queued for pit stop: Tire -> Soft, Fuel -> 50.0%"
}
```

## Error Responses

All endpoints may return the following error responses:

### Bad Request (400)

```json
{
  "status": "error",
  "code": 400,
  "message": "Invalid parameters: ..."
}
```

### Not Found (404)

```json
{
  "status": "error",
  "code": 404,
  "message": "Car number 999 not found."
}
```

### Server Error (500)

```json
{
  "status": "error",
  "code": 500,
  "message": "Internal server error"
}
```

## Websocket Updates

Real-time race updates are available via websocket connection:

**Endpoint:** `wss://api.racing-sim.com/v1/live`

**Events:**
- `race_update`: General race status updates
- `car_update`: Individual car status updates
- `pit_stop`: Pit stop events
- `race_finished`: Race completion event 
import cv2 as cv
import mediapipe as mp
import numpy as np
import pyautogui
import threading
import queue
import time

# MediaPipe Hand Landmarker setup
from mediapipe.tasks import python
from mediapipe.tasks.python import vision
BaseOptions = mp.tasks.BaseOptions
HandLandmarker = mp.tasks.vision.HandLandmarker
HandLandmarkerOptions = mp.tasks.vision.HandLandmarkerOptions
HandLandmarkerResult = mp.tasks.vision.HandLandmarkerResult
VisionRunningMode = mp.tasks.vision.RunningMode

# Get screen dimensions
screenWidth, screenHeight = pyautogui.size()

# Create thread-safe queues
mouse_queue = queue.Queue()
click_queue = queue.Queue()
key_queue = queue.Queue()

class SmoothMouseController:
    def __init__(self, smoothing=0.5, speed_factor=2.0):
        """
        Initialize smooth mouse controller
        :param smoothing: Smoothing factor (0-1, lower = smoother but slower)
        :param speed_factor: Multiplier for cursor speed
        """
        self.current_x, self.current_y = pyautogui.position()
        self.smoothing = smoothing
        self.speed_factor = speed_factor
        self.last_click_time = 0
        self.click_cooldown = 0.5  # Prevent rapid successive clicks
        self.last_key_time = 0
        self.key_cooldown = 0.5  # Prevent rapid key presses

    def update_position(self, target_x, target_y):
        """
        Smoothly move the cursor towards the target position
        """
        # Invert Y-axis to match screen coordinates
        target_y = screenHeight - target_y

        # Calculate smooth interpolation
        new_x = (1 - self.smoothing) * target_x + self.smoothing * self.current_x
        new_y = (1 - self.smoothing) * target_y + self.smoothing * self.current_y

        # Apply speed factor
        dx = (new_x - self.current_x) * self.speed_factor
        dy = (new_y - self.current_y) * self.speed_factor

        # Update current position
        self.current_x += dx
        self.current_y += dy

        # Ensure cursor stays within screen bounds
        self.current_x = max(0, min(self.current_x, screenWidth))
        self.current_y = max(0, min(self.current_y, screenHeight))

        # Move the cursor
        pyautogui.moveTo(self.current_x, self.current_y)

    def try_click(self):
        """
        Perform a click with a cooldown to prevent spam
        """
        current_time = time.time()
        if current_time - self.last_click_time > self.click_cooldown:
            pyautogui.click()
            self.last_click_time = current_time

    def try_key_press(self, key):
        """
        Perform a key press with a cooldown to prevent spam
        """
        current_time = time.time()
        if current_time - self.last_key_time > self.key_cooldown:
            pyautogui.press(key)
            self.last_key_time = current_time

# Initialize smooth mouse controller
mouse_controller = SmoothMouseController()

def mouse_control_thread():
    """
    Separate thread to handle smooth mouse movement, clicking, and key presses
    """
    while True:
        try:
            # Handle mouse movement
            if not mouse_queue.empty():
                landmarks = mouse_queue.get()
                mouse_controller.update_position(
                    landmarks['mouse_x'], 
                    landmarks['mouse_y']
                )
            
            # Handle clicking
            if not click_queue.empty():
                click_queue.get()
                mouse_controller.try_click()
            
            # Handle key presses
            if not key_queue.empty():
                key = key_queue.get()
                mouse_controller.try_key_press(key)
            
            time.sleep(0.01)  # Prevent tight looping
        except Exception as e:
            print(f"Mouse control thread error: {e}")
            break

def is_hand_closed(landmarks):
    """
    Determine if hand is closed by checking if fingertips are bent towards palm
    """
    # Landmarks for fingertips
    tip_landmarks = [8, 12, 16, 20]  # Index, Middle, Ring, Pinky
    
    # Landmarks for middle joints of same fingers
    mid_landmarks = [7, 11, 15, 19]
    
    # Landmark for wrist and base of hand
    wrist = landmarks[0]
    
    # Count how many fingers are bent
    bent_fingers = 0
    for tip, mid in zip(tip_landmarks, mid_landmarks):
        # Check if fingertip is below the middle joint relative to the wrist
        if landmarks[tip].y > landmarks[mid].y:
            bent_fingers += 1
    
    # Consider hand closed if 3 or 4 fingers are bent
    return bent_fingers >= 3

def callback_fn(result, output_image, timestamp_ms):
    """
    Callback function that handles mouse movement, clicking, and gesture detection
    """
    if result.hand_landmarks and result.handedness:
        for idx, landmarks in enumerate(result.hand_landmarks):
            # Determine hand side (left or right)
            # MediaPipe returns handedness with probability
            handedness = result.handedness[idx][0]
            
            # Only process right hand for mouse and click
            if handedness.category_name.lower() == 'right':
                # Cursor movement (using index finger)
                index_finger = landmarks[8]
                
                # Map normalized coordinates to screen coordinates
                # Flip x-coordinate to match natural hand movement
                mouse_x = int((1 - index_finger.x) * screenWidth)
                mouse_y = int((1 - index_finger.y) * screenHeight)
                
                # Put movement coordinates in queue
                try:
                    # Clear any existing movement items
                    while not mouse_queue.empty():
                        mouse_queue.get()
                    mouse_queue.put({
                        'mouse_x': mouse_x, 
                        'mouse_y': mouse_y
                    })
                except Exception as e:
                    print(f"Error in movement queue: {e}")
                
                # Pinch-to-click (thumb tip and middle finger tip)
                thumb_tip = landmarks[4]
                middle_finger_tip = landmarks[11]
                
                # Calculate distance between thumb and middle finger tips
                distance = np.sqrt(
                    (thumb_tip.x - middle_finger_tip.x)**2 + 
                    (thumb_tip.y - middle_finger_tip.y)**2
                )
                
                # If tips are close, trigger a click
                if distance < 0.05:
                    try:
                        click_queue.put(True)
                    except Exception as e:
                        print(f"Error in click queue: {e}")
            
            # Left hand gesture detection remains the same
            if handedness.category_name.lower() == 'left':
                # Detect open or closed hand
                if is_hand_closed(landmarks):
                    # Closed hand - press '2'
                    try:
                        key_queue.put('2')
                    except Exception as e:
                        print(f"Error in key queue: {e}")
                else:
                    # Open hand - press '1'
                    try:
                        key_queue.put('1')
                    except Exception as e:
                        print(f"Error in key queue: {e}")

# Path to the pre-trained model
model_path = './hand_landmarker.task'

# Set up hand landmarker options
options = HandLandmarkerOptions(
    base_options=BaseOptions(model_asset_path=model_path),
    running_mode=VisionRunningMode.LIVE_STREAM,
    num_hands=2,  # Allow tracking of both hands
    min_hand_detection_confidence=0.5,
    min_hand_presence_confidence=0.5,
    min_tracking_confidence=0.5,
    result_callback=callback_fn
)

# Start the mouse control thread
mouse_thread = threading.Thread(target=mouse_control_thread, daemon=True)
mouse_thread.start()

# OpenCV VideoCapture setup
cap = cv.VideoCapture(0)
if not cap.isOpened():
    print("Cannot open camera")
    exit()

try:
    with HandLandmarker.create_from_options(options) as hand_landmarker:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Can't receive frame (stream end?). Exiting...")
                break
            
            # Convert frame to RGB
            frame_rgb = cv.cvtColor(frame, cv.COLOR_BGR2RGB)
            
            # Convert OpenCV frame to MediaPipe Image
            mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=frame_rgb)
            
            # Send image to Hand Landmarker for asynchronous processing
            hand_landmarker.detect_async(mp_image, int(cap.get(cv.CAP_PROP_POS_MSEC)))
            
            if cv.waitKey(1) == ord('q'):
                break

except Exception as e:
    print(f"An error occurred: {e}")

finally:
    # Release resources
    cap.release()
    cv.destroyAllWindows()

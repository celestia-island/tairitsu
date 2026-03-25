// @ts-nocheck
import { wasmExports } from "./wasm-exports";

globalThis.__nextListenerId = globalThis.__nextListenerId || 1n;
globalThis.__listeners = globalThis.__listeners || new Map();
globalThis.__eventHandles = globalThis.__eventHandles || new Map();
globalThis.__nextEventHandle = globalThis.__nextEventHandle || 1n;

export const event_target_exports = {
  addEventListener(target, eventType, useCapture) {
    try {
      const element = globalThis.__elementHandles.get(target);
      if (!element) {
        return "Target handle " + target + " not found";
      }

      const listener = function (event) {
        // Store the event for later access
        const eventHandle = globalThis.__nextEventHandle++;
        globalThis.__eventHandles.set(eventHandle, event);

        // Dispatch to WASM component if exports are available
        if (wasmExports && wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"]) {
          const callbacks = wasmExports["tairitsu-browser:full/event-callbacks@0.2.0"];
          const listeners = globalThis.__listeners;

          // Find the listener ID for this element and event type
          let listenerId = 0n;
          for (const [id, info] of listeners) {
            if (info.target === element && info.type === eventType) {
              listenerId = id;
              break;
            }
          }

          if (listenerId !== 0n) {
            // Determine event type and call appropriate callback
            if (eventType === "click" || eventType === "mousedown" || eventType === "mouseup" ||
              eventType === "mousemove" || eventType === "mouseenter" || eventType === "mouseleave") {
              if (callbacks.on_mouse_event) {
                const mouseEvent = event;
                callbacks.on_mouse_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    client_x: mouseEvent.clientX,
                    client_y: mouseEvent.clientY,
                    screen_x: mouseEvent.screenX,
                    screen_y: mouseEvent.screenY,
                    offset_x: mouseEvent.offsetX,
                    offset_y: mouseEvent.offsetY,
                    page_x: mouseEvent.pageX,
                    page_y: mouseEvent.pageY,
                    movement_x: mouseEvent.movementX,
                    movement_y: mouseEvent.movementY,
                    button: mouseEvent.button || 0,
                    buttons: mouseEvent.buttons || 0,
                    ctrl_key: mouseEvent.ctrlKey || false,
                    shift_key: mouseEvent.shiftKey || false,
                    alt_key: mouseEvent.altKey || false,
                    meta_key: mouseEvent.metaKey || false,
                  }
                );
              }
            } else if (eventType === "keydown" || eventType === "keyup" || eventType === "keypress") {
              if (callbacks.on_keyboard_event) {
                const keyboardEvent = event;
                callbacks.on_keyboard_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    key: keyboardEvent.key || "",
                    code: keyboardEvent.code || "",
                    key_code: keyboardEvent.keyCode || 0,
                    ctrl_key: keyboardEvent.ctrlKey || false,
                    shift_key: keyboardEvent.shiftKey || false,
                    alt_key: keyboardEvent.altKey || false,
                    meta_key: keyboardEvent.metaKey || false,
                    repeat: keyboardEvent.repeat || false,
                  }
                );
              }
            } else if (eventType === "focus" || eventType === "blur") {
              if (callbacks.on_focus_event) {
                callbacks.on_focus_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    related_target: event.relatedElement ?
                      (globalThis.__elementHandles.get(event.relatedElement) || 0n) : undefined,
                  }
                );
              }
            } else if (eventType === "input") {
              if (callbacks.on_input_event) {
                const inputEvent = event;
                callbacks.on_input_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    data: inputEvent.data,
                    input_type: inputEvent.inputType || "",
                  }
                );
              }
            } else if (eventType === "wheel") {
              if (callbacks.on_wheel_event) {
                const wheelEvent = event;
                callbacks.on_wheel_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    delta_x: wheelEvent.deltaX || 0,
                    delta_y: wheelEvent.deltaY || 0,
                    delta_z: wheelEvent.deltaZ || 0,
                    delta_mode: wheelEvent.deltaMode || 0,
                    client_x: wheelEvent.clientX || 0,
                    client_y: wheelEvent.clientY || 0,
                    screen_x: wheelEvent.screenX || 0,
                    screen_y: wheelEvent.screenY || 0,
                    ctrl_key: wheelEvent.ctrlKey || false,
                    shift_key: wheelEvent.shiftKey || false,
                    alt_key: wheelEvent.altKey || false,
                    meta_key: wheelEvent.metaKey || false,
                  }
                );
              }
            } else if (eventType === "touchstart" || eventType === "touchmove" || eventType === "touchend" || eventType === "touchcancel") {
              if (callbacks.on_touch_event) {
                const touchEvent = event;
                const toTouchPoint = (touch) => ({
                  identifier: touch.identifier || 0,
                  client_x: touch.clientX || 0,
                  client_y: touch.clientY || 0,
                  screen_x: touch.screenX || 0,
                  screen_y: touch.screenY || 0,
                  page_x: touch.pageX || 0,
                  page_y: touch.pageY || 0,
                  target: target,
                  force: touch.force || 0,
                  radius_x: touch.radiusX || 0,
                  radius_y: touch.radiusY || 0,
                  rotation_angle: touch.rotationAngle || 0,
                });
                callbacks.on_touch_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    touches: Array.from(touchEvent.touches || []).map(toTouchPoint),
                    changed_touches: Array.from(touchEvent.changedTouches || []).map(toTouchPoint),
                    target_touches: Array.from(touchEvent.targetTouches || []).map(toTouchPoint),
                    timestamp: touchEvent.timeStamp || 0,
                  }
                );
              }
            } else if (eventType === "pointerdown" || eventType === "pointerup" || eventType === "pointermove" ||
              eventType === "pointercancel" || eventType === "pointerout" || eventType === "pointerleave" ||
              eventType === "pointerover" || eventType === "pointerenter" || eventType === "gotpointercapture" ||
              eventType === "lostpointercapture") {
              if (callbacks.on_pointer_event) {
                const pointerEvent = event;
                callbacks.on_pointer_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    pointer_id: pointerEvent.pointerId || 0,
                    pointer_type: pointerEvent.pointerType || "mouse",
                    is_primary: pointerEvent.isPrimary || false,
                    client_x: pointerEvent.clientX || 0,
                    client_y: pointerEvent.clientY || 0,
                    screen_x: pointerEvent.screenX || 0,
                    screen_y: pointerEvent.screenY || 0,
                    offset_x: pointerEvent.offsetX || 0,
                    offset_y: pointerEvent.offsetY || 0,
                    page_x: pointerEvent.pageX || 0,
                    page_y: pointerEvent.pageY || 0,
                    movement_x: pointerEvent.movementX || 0,
                    movement_y: pointerEvent.movementY || 0,
                    width: pointerEvent.width || 0,
                    height: pointerEvent.height || 0,
                    pressure: pointerEvent.pressure || 0,
                    tangential_pressure: pointerEvent.tangentialPressure || 0,
                    tilt_x: pointerEvent.tiltX || 0,
                    tilt_y: pointerEvent.tiltY || 0,
                    twist: pointerEvent.twist || 0,
                    button: pointerEvent.button || 0,
                    buttons: pointerEvent.buttons || 0,
                    ctrl_key: pointerEvent.ctrlKey || false,
                    shift_key: pointerEvent.shiftKey || false,
                    alt_key: pointerEvent.altKey || false,
                    meta_key: pointerEvent.metaKey || false,
                  }
                );
              }
            } else if (eventType === "transitionend") {
              if (callbacks.on_transition_event) {
                const transitionEvent = event;
                callbacks.on_transition_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    property_name: transitionEvent.propertyName || "",
                    elapsed_time: transitionEvent.elapsedTime || 0,
                    pseudo_element: transitionEvent.pseudoElement || "",
                  }
                );
              }
            } else if (eventType === "animationstart" || eventType === "animationend" || eventType === "animationiteration") {
              if (callbacks.on_animation_event) {
                const animationEvent = event;
                callbacks.on_animation_event(
                  listenerId,
                  eventHandle,
                  {
                    target: target,
                    animation_name: animationEvent.animationName || "",
                    pseudo_element: animationEvent.pseudoElement || "",
                    elapsed_time: animationEvent.elapsedTime || 0,
                    iteration: animationEvent.iteration || 0,
                  }
                );
              }
            } else {
              if (callbacks.on_generic_event) {
                callbacks.on_generic_event(listenerId, eventHandle, eventType);
              }
            }
          }
        }
      };

      element.addEventListener(eventType, listener, useCapture);

      const listenerId = globalThis.__nextListenerId++;
      globalThis.__listeners.set(listenerId, {
        target: element,
        type: eventType,
        listener: listener,
        useCapture: useCapture,
      });

      return listenerId;
    } catch (e) {
      return String(e);
    }
  },
  removeEventListener(_target, listenerId) {
    try {
      const info = globalThis.__listeners.get(listenerId);
      if (!info) {
        return "Listener ID " + listenerId + " not found";
      }
      info.target.removeEventListener(info.type, info.listener, info.useCapture);
      globalThis.__listeners.delete(listenerId);
    } catch (e) {
      return String(e);
    }
  },
  preventDefault(event) {
    globalThis.__eventHandles.get(event)?.preventDefault();
  },
  stopPropagation(event) {
    globalThis.__eventHandles.get(event)?.stopPropagation();
  },
};

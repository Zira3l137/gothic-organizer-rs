# How to Add a New Window

To add a new window to the application, follow these steps:

1. **Add a new variant**  
   Extend the `crate::app::window::ApplicationWindow` enum with a new variant and `#[display("...")]` attribute.

2. **Update string conversion**  
   Add the variant to the `From<&str>` `match` branch.

3. **Register in `into_iter`**  
   Insert the new variant into the static `WINDOWS` array and adjust its length.

4. **Define defaults**  
   Add entries for the new variant in:
   - `default_size`  
   - `default_position`  

5. **Map to a view**  
   In the `view` function, add a match branch calling the corresponding `*_view(app)` function.

6. **Implement the view**  
   Create a new module in `crate::gui::<name>` with a function returning an `iced::Element<'a, Message>`.

---

✅ **In short:**  
**Enum → String conversion → Iter list → Defaults → View → Module**


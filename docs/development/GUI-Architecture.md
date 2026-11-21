# GUI Architecture – Recursive Separation of Concerns with Feature-Oriented Component Composition

## 1. Core architectural principle

The GUI architecture must follow Recursive Separation of Concerns with Feature-Oriented Component Composition.

All GUI elements must be organized around cohesive features and composed of small, independently understandable components. Separation of Concerns must be applied recursively at every level of the UI hierarchy, from complete screens down to individual non-trivial components.

The following principles are mandatory:

- **Separation of Concerns (SoC):** Keep distinct responsibilities separate, including within the presentation layer itself.
- **Single Responsibility Principle (SRP):** Every component must have one clearly defined reason to change.
- **Single Level of Abstraction Principle (SLAP):** Components and methods must operate at a consistent level of abstraction.
- **Interface Segregation Principle (ISP):** Components expose only the properties, events, and capabilities their consumers require.
- **Dependency Inversion Principle (DIP):** Presentation logic must not depend directly on concrete infrastructure implementations.
- **Composition over Inheritance:** Construct complex interfaces by composing focused components.

These principles apply regardless of the chosen language, GUI framework, or rendering technology.

## 2. Recursive component decomposition

Every logically independent, non-trivial UI element must be implemented as a dedicated component, widget, view, control, or equivalent framework construct.

A typical UI hierarchy is: Page → Sections → Components → Primitive UI Elements.

At every level:

- Components must focus on a single responsibility.
- Parent components primarily compose and coordinate their children.
- Implementation details must be delegated to lower-level components.
- Components must expose minimal and explicit interfaces.
- Internal implementation details must remain encapsulated.
- Components must be independently understandable, modifiable, and testable where practical.

Component decomposition must be driven by responsibilities and abstraction boundaries, not arbitrary file sizes or line counts. Trivial UI primitives do not require separate components unless doing so provides meaningful encapsulation, reuse, or consistency.

## 3. Feature-oriented organization

Organize GUI code primarily around features and cohesive functional responsibilities, rather than large global collections of screens, widgets, controllers, or view models.

Each feature should own its relevant:

- Screens and UI components
- Presentation state and state management
- Presentation-specific transformations
- User interaction coordination
- Feature-specific styles and visual resources

Avoid unnecessary dependencies between features. Shared UI components should be extracted only when they represent genuinely reusable presentation concepts. Do not move feature-specific behavior into global components or services merely to facilitate reuse.

## 4. Separation of presentation responsibilities

The GUI must distinguish between:

- **Visual presentation:** Rendering, layout, styling, animations, and visual feedback.
- **Presentation state:** UI-specific state, selections, loading indicators, visibility, and interaction state.
- **Interaction coordination:** Translating user actions into application requests and presenting their results.
- **Application logic:** Use cases and application workflows, implemented outside the presentation layer.
- **Infrastructure:** Networking, persistence, hardware access, and platform-specific integrations, implemented outside the presentation layer.

A component must not combine unrelated responsibilities from these categories. Local, purely visual interaction state may remain within a component when appropriate. Complex or shared state transitions must be delegated to dedicated, narrowly scoped state-management components.

## 5. No monolithic UI components or controllers

Monolithic screens, widgets, views, view models, controllers, and state managers are prohibited. Specifically:

- Screens must not implement the internal behavior of unrelated child components.
- Rendering methods must not contain multiple unrelated UI sections with their own substantial logic.
- Controllers and state managers must not accumulate unrelated responsibilities.
- Event handlers must not implement complex application workflows.
- UI components must not directly access infrastructure services.
- Avoid catch-all controllers, generic UI managers, and centralized objects controlling unrelated features.
- Do not extend an existing component with a new responsibility merely because it is convenient.

When a component begins handling multiple independent concerns, decompose it into cohesive collaborators. A component being technically manageable in size does not justify mixing responsibilities.

## 6. Single level of abstraction

Every UI component and method must operate at a consistent level of abstraction. High-level components should express the structure and intent of the interface rather than the implementation details of their children.

For example, a device management screen should compose elements such as:

- Device discovery panel
- Available devices list
- Connection status
- Device settings

It should not directly implement discovery animations, individual device-row layouts, connection workflows, and settings validation in one rendering method. The same principle must be applied recursively within each of these components. Extract lower-level implementation details into appropriately named components or focused helper functions.

## 7. Component contracts and data flow

Each component must expose a minimal, explicit contract.

- Pass only the data required by the component.
- Prefer immutable inputs and explicit outputs.
- Use callbacks, commands, events, or framework-specific mechanisms for communicating user actions.
- Prefer unidirectional data flow where supported.
- Avoid hidden dependencies and mutable global state.
- Do not allow child components to manipulate unrelated application state.
- Keep sibling components loosely coupled.
- Avoid passing large state objects when a component only requires a small subset.

Component contracts should express intent rather than expose internal implementation details.

## 8. State management boundaries

State management must follow the same recursive separation principles as visual components.

- Distinguish local component state from feature-level and application-wide state.
- Keep state managers narrowly scoped to cohesive responsibilities.
- Avoid monolithic controllers managing an entire screen or multiple unrelated features.
- Delegate business rules and workflows to application-layer services or use cases.
- Prefer explicit state transitions and predictable data flow.
- Keep asynchronous operations and error states explicit.
- Avoid unnecessary duplication or synchronization of state.

The choice of state-management framework must not dictate the boundaries of responsibilities.

## 9. Styling and visual consistency

Visual styling must also respect component boundaries.

- Centralize genuinely shared design tokens such as colors, typography, spacing, and dimensions.
- Encapsulate component-specific styling within the corresponding component or feature.
- Prefer reusable visual primitives and shared styles over duplicated styling rules.
- Avoid global styles that introduce unintended coupling between unrelated features.
- Keep styling and layout concerns separate from business logic.
- Ensure that themes and visual variants can be changed without modifying application behavior.

## 10. Avoid overengineering

Recursive decomposition does not mean creating a separate component for every primitive UI element. Do not introduce abstractions solely to satisfy architectural patterns.

A new component is justified when it provides at least one meaningful benefit:

- Encapsulates a distinct responsibility.
- Establishes a useful abstraction boundary.
- Separates different levels of abstraction.
- Contains independently meaningful behavior or state.
- Improves testability or maintainability.
- Represents a genuinely reusable UI concept.

Prefer cohesive components over both monolithic implementations and excessive fragmentation.

## 11. Architectural quality gate

Before completing any GUI implementation, perform an explicit architectural review. Verify:

1. **Recursive SoC:** Are responsibilities separated at every relevant level of the component hierarchy?
2. **Feature orientation:** Is the implementation organized around cohesive features?
3. **Component composition:** Do higher-level components primarily compose lower-level components?
4. **SRP:** Does each component have one clearly defined responsibility?
5. **SLAP:** Are abstraction levels consistent within components and methods?
6. **Encapsulation:** Are implementation details hidden behind minimal component interfaces?
7. **State boundaries:** Are local, feature-level, and application-wide states appropriately separated?
8. **Dependency boundaries:** Are presentation components independent of infrastructure implementations?
9. **Modularity:** Can components be understood, modified, and tested independently where practical?
10. **Complexity:** Are there monolithic components, excessive coupling, or unnecessary abstractions?

Refactor architectural violations before considering the implementation complete.

## 12. Implementation rule for AI coding agents

When implementing or modifying a GUI feature:

1. Identify the distinct presentation responsibilities.
2. Define the intended component hierarchy and ownership of state.
3. Reuse existing components when their responsibilities match.
4. Introduce new components when meaningful responsibility or abstraction boundaries require them.
5. Implement behavior through composition and explicit component contracts.
6. Review the resulting implementation for mixed responsibilities, abstraction-level violations, and excessive coupling.
7. Refactor architectural violations before declaring the task complete.

For non-trivial GUI changes, briefly describe the intended component decomposition before implementation. Do not sacrifice architectural boundaries for implementation speed or convenience.

## Guiding rule

Build every GUI as a recursively composed hierarchy of cohesive, feature-oriented components. Each component must have a single responsibility, operate at a consistent level of abstraction, expose a minimal contract, and delegate unrelated concerns to dedicated collaborators.

Apply these principles recursively, not merely at the boundary between GUI and application logic.

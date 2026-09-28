# Traffic-Jam

## Game Design Document

**Status:** Initial Design
**Working Title:** Traffic-Jam
**Genre:** Transportation-focused city builder / traffic engineering simulation
**Primary Platform:** Linux
**Secondary Target:** Steam Deck
**Engine:** Rust + Bevy
**Perspective:** Top-down vector map
**Primary Focus:** Traffic optimization, civil engineering, city growth, citizen happiness
**Design Priority:** Simulation scale first, graphical fidelity second

---

# 1. Game Vision

**Traffic-Jam** is a transportation-focused city builder where the player acts primarily as the city's civil engineer rather than an all-powerful mayor.

The player builds and modifies the transportation network while the city develops around it.

The player controls:

* Roads
* Intersections
* Traffic signals
* Stop and yield controls
* Highways
* Bridges
* Tunnels
* Bus routes
* Rail
* Subway
* Parking
* Zoning
* Traffic control
* Construction projects
* Infrastructure maintenance
* Emergency traffic responses
* Transportation-focused ordinance proposals

The player does **not** manually place normal buildings.

Instead, roads create developable addresses and the player zones street frontage. Private development then responds to transportation access, density limits, land value, demand, services, taxation, and neighborhood desirability.

The core fantasy is:

**Factorio-level optimization, except the machine is a living city.**

The goal is to create an increasingly large city where people, vehicles, freight, transit, emergency services, and businesses can move efficiently while maintaining citizen happiness and safety.

A city with high throughput but miserable citizens is not a perfect city.

A city with happy residents but gridlocked roads is also not a perfect city.

The player's challenge is to find the balance.

---

# 2. Core Design Pillars

## 2.1 Flow Is the Game

Traffic flow is the primary system.

The player should constantly be asking:

* Where is traffic coming from?
* Where is it going?
* Why is this intersection failing?
* Why are drivers choosing this road?
* Why is this parking lot full?
* Why are buses late?
* Why are trucks using this neighborhood?
* Why did this redesign move congestion somewhere else?
* Can the same movement happen with fewer vehicles?
* Can this network survive an accident or closure?

Traffic must arise from simulated behavior rather than arbitrary traffic generation wherever practical.

---

## 2.2 The City Is Alive

Citizens:

* Live somewhere.
* Work somewhere.
* Have schedules.
* Own vehicles.
* Use transit.
* Search for parking.
* Remember bad commutes.
* Try alternative routes.
* Become impatient.
* Respond to enforcement.
* Change jobs.
* Move homes.
* Leave the city.
* Change transportation methods.

Businesses create employment and traffic.

Schools create morning and afternoon traffic.

Stadiums create event surges.

Industrial areas generate trucks.

Neighborhoods develop identities.

The city should feel like a system reacting to the player's engineering decisions.

---

## 2.3 Easy to Build, Deep to Engineer

A new player should be able to:

1. Draw a road.
2. Zone it.
3. Watch development occur.
4. See traffic appear.
5. Notice a problem.
6. Make an improvement.

Advanced players can then dig into:

* Individual signal phases
* Turn lanes
* Corridor synchronization
* Parking restrictions
* Construction phases
* Scheduled detours
* Event traffic plans
* Transit frequency
* Police placement
* Road maintenance
* Detailed analytics

Most complex systems should support:

**Manual / AI Assisted / Fully Automated**

where appropriate.

---

# 3. Primary Gameplay Loop

The repeating game loop is:

1. **Observe**

   * Watch traffic.
   * Read social posts.
   * Inspect alerts.
   * Review analytics.
   * Identify inefficient areas.

2. **Diagnose**

   * Inspect roads.
   * Inspect intersections.
   * Review vehicle movement.
   * Review congestion heatmaps.
   * Determine why the problem exists.

3. **Design**

   * Modify roads.
   * Adjust intersections.
   * Modify transit.
   * Add parking.
   * Create detours.
   * Change signal timings.
   * Propose ordinances.

4. **Test**

   * Enter Planning Mode.
   * Instantly simulate the proposed network.
   * Compare current versus proposed conditions.

5. **Approve**

   * Review cost.
   * Review construction time.
   * Review expected disruption.
   * Commit the project.

6. **Construct**

   * Crews physically modify infrastructure over time.
   * Roads and lanes may close.
   * Traffic adapts.

7. **Measure**

   * Compare before/after performance.
   * Watch citizen reaction.
   * Track economic consequences.

8. **Optimize Again**

The city grows and continuously creates new bottlenecks.

---

# 4. Game Modes

## 4.1 Endless City

Primary initial mode.

There is no final victory condition.

The player can operate the city indefinitely.

Long-term achievement goals recognize exceptional performance.

Examples:

* 30 days without a major traffic jam
* 95%+ transportation efficiency
* Average commute below a target
* Zero fatal crashes for one year
* Very high emergency response performance
* 90%+ citizen happiness
* Extremely high transit reliability

---

## 4.2 Scenario Mode

Future mode.

Scenarios present existing cities with predefined engineering problems.

Examples:

* Repair a highway-dependent city.
* Fix a congested downtown.
* Prepare a stadium district.
* Recover from bridge failure.
* Modernize a suburban network.
* Handle rapid population growth.

---

## 4.3 Multiplayer

**Out of scope for the initial release.**

Architecture should remain compatible with future:

* Hosted servers
* Player-hosted servers
* Competitive cities
* Regional interaction
* Disruptive events
* Population races
* Performance competitions

The initial simulation does not need deterministic multiplayer lockstep.

---

# 5. Time Simulation

Supported simulation speeds:

* **Pause**
* **1×**
* **2×**
* **4×**
* **8×**

Target conceptual speeds:

* 1× = detailed real-time simulation
* 2× = approximately 1 game hour per minute
* 4× = approximately 1 game day per minute
* 8× = approximately 1 game week per minute

Exact rates may be adjusted for computational practicality and gameplay.

The simulation includes:

* 24-hour cycle
* Weekdays
* Weekends
* Seasons
* Weekly budgets
* Monthly ordinance cycles
* Monthly growth reports
* Annual mayor elections

At high simulation speed, unnecessary visual and agent-level calculations should be reduced.

---

# 6. City Scale

Typical expected city population:

**10,000–100,000 residents**

The architecture must support substantially larger cities.

Ultimate stress-test target:

**10,000,000 population**

A 10-million-person city does not mean ten million fully simulated visible entities.

Simulation Level of Detail is mandatory.

---

# 7. Simulation Level of Detail

## Near Camera

Use detailed simulation.

Potentially include:

* Individual vehicles
* Individual pedestrians
* Detailed intersections
* Lane behavior
* Parking search
* Signal phases

## Active District

Use simplified but route-aware agents.

## Distant District

Aggregate:

* Traffic flow
* Population trips
* Transit demand
* Freight
* Parking
* Economic effects

## Massive City

Citizens become statistical populations unless individual simulation is necessary.

This system is fundamental to reaching the 10-million population stress target.

---

# 8. Citizen Simulation

Citizens conceptually have:

* Home
* Job
* Salary
* Age
* Household
* Income
* Vehicle ownership
* Preferred transportation method
* Work schedule
* Patience
* Risk tolerance
* Happiness
* Commute satisfaction
* Parking preference
* Likelihood of moving

Most citizens do not require permanently instantiated detailed objects.

Population can normally exist statistically.

When the player selects a citizen, the game may generate a persistent individual representation based on the underlying population.

Example:

**Emily Carter**

* Home: 118 Oak Street
* Job: Downtown Medical Center
* Income: $61,000
* Commute: 24 minutes
* Vehicle: EV
* Typical Route: Oak → Chester → Downtown
* Happiness: 76%
* Commute Satisfaction: 62%

Selected citizens can remain detailed while relevant.

Once they move away or die, their detailed representation can be deleted.

Only a relatively small number of player-inspected citizens need this level of persistence.

---

# 9. Citizen Route Behavior

Drivers initially calculate a route using current known conditions.

They behave similarly to drivers following GPS.

If conditions change after departure, behavior differs.

Initial behavioral target:

* 10% highly responsive
* 50% patient
* 25% impatient
* 15% risky

Only approximately 1–2% of normal drivers should routinely break traffic laws.

These values may be tuned during development.

Driver behavior changes over time.

The longer someone waits, the more likely they are to seek an alternative.

Drivers remember routes that performed poorly.

Their route memory experiences **weekly decay**.

This allows a driver to reconsider a previously bad route after conditions may have improved.

The network therefore does not instantly reach mathematical equilibrium.

It gradually adapts.

---

# 10. Specialized Driver Behavior

## City Vehicles

Generally patient.

## Buses

Patient and route-bound.

## Semi-Trucks

Patient.

Prefer major roads.

Avoid:

* Tight residential streets
* Low clearances
* Incompatible underpasses
* Sharp geometry
* Restricted routes

## Taxi and Rideshare

More impatient than average.

## Emergency Vehicles

Can:

* Exceed speed limits
* Trigger traffic to pull over
* Receive priority
* Cause crashes themselves

## Risky Drivers

More likely to:

* Speed
* Make aggressive lane changes
* Run lights
* Roll stop signs
* Attempt unusual detours

---

# 11. Jobs and Housing Decisions

Citizens initially favor jobs within approximately **5 miles of home**.

They then consider:

* Salary
* Travel time
* Transportation availability
* Job availability

Higher-paying jobs can justify longer commutes.

When housing becomes available closer to a high-paying job, a citizen may relocate.

This creates an evolving relationship between:

**employment → transportation → housing → land value**

---

# 12. Roads

Road creation uses a hybrid system.

The player:

1. Draws the road.
2. Selects the road segment.
3. Configures engineering details.

Supported road concepts include:

* Alley/service road
* Local street
* Collector
* Arterial
* Boulevard
* Rural road
* Industrial/freight road
* Highway
* Limited-access freeway

Road class defines sensible defaults but remains customizable.

---

# 13. Road Editor

Basic workflow:

**Click start → draw → click end**

Optional snapping:

* Roads
* Intersections
* Grid alignment
* Parallel roads
* 45° angles
* 90° angles
* Property boundaries

Curved roads are supported.

Advanced users may disable snapping.

---

# 14. Elevation

Terrain simulation is intentionally limited.

The game is not a terrain engineering simulator.

Road infrastructure can exist in layers:

* Surface
* Bridge/elevated
* Underground/tunnel

Grade can still matter for road design and heavy vehicles.

The goal is functional transportation engineering without requiring complex terrain modeling.

---

# 15. Intersections

Crossing roads automatically create an intersection unless explicitly configured as:

* Overpass
* Underpass
* No connection

Intersection controls include:

* Through lanes
* Left-turn lanes
* Right-turn lanes
* Protected left turns
* Yield movements
* Turn restrictions
* Pedestrian crossings
* Stop signs
* Yield signs
* Traffic signals
* Roundabouts

---

# 16. Traffic Signals

Advanced signal configuration includes:

* Cycle length
* Signal phases
* Green duration
* Yellow duration
* All-red duration
* Protected turns
* Pedestrian timing
* Transit priority
* Emergency preemption
* Traffic detectors
* Time-based schedules
* Event schedules
* Adaptive timing

Beginners can use AI optimization.

Advanced players can manually configure everything.

---

# 17. Corridor Synchronization

Players can coordinate several intersections.

Capabilities include:

* Green-wave corridors
* Direction preference
* Target travel speed
* Time-dependent signal plans

Optimizing the main corridor may negatively affect side streets.

This tradeoff is intentional.

---

# 18. Stop and Yield Control

Supported:

* Two-way stop
* Three-way stop
* Four-way stop
* Yield
* Uncontrolled intersection

Drivers should negotiate realistic right-of-way instead of simply stopping for an arbitrary timer.

Risky drivers may violate controls.

---

# 19. Roundabouts

Roundabouts support:

* Multiple lanes
* Entry configuration
* Exit configuration
* Yield control
* Pedestrian crossings
* Truck apron
* Speed configuration

Poorly designed roundabouts can perform poorly.

---

# 20. Signal Failures

Signals can fail because of:

* Equipment failure
* Weather
* Collision
* Construction
* Power outage

Player responses include:

* Dispatch repair crew
* Add police traffic control
* Temporarily close lanes
* Create a detour
* Add temporary traffic control

---

# 21. Zoning

The player zones **street frontage**, not individual arbitrary tiles.

Road construction creates:

* Street sections
* Addresses
* Adjacent developable land

A zoning designation applies to the land touching a road segment between intersections.

No land is conceptually ownerless.

Unzoned property can represent:

* Park
* Field
* City-owned property
* Open land

Different open-land types affect nearby value differently.

---

# 22. Zone Types

Core zones:

* Residential
* Commercial
* Office
* Industrial
* Farm

Special or higher-detail categories can include:

* Light industrial
* Heavy industrial
* Manufacturing
* Warehousing/logistics
* High-tech industry

---

# 23. Density

The player assigns:

* Zone type
* Optional maximum density

The market determines actual density.

Density responds to:

* Land value
* Accessibility
* Demand
* Transportation
* Services
* Taxes
* Neighborhood desirability

Higher density is represented visually through increased color saturation.

No detailed building rendering is required.

---

# 24. Mixed Use

Dense development can naturally transition into combinations such as:

* Residential + Commercial
* Residential + Office
* Commercial + Office
* Residential + Commercial + Office

Mixed-use neighborhoods reduce some transportation demand by bringing destinations closer together.

---

# 25. Special Buildings

Special facilities are represented as functional zones rather than detailed structures.

Examples:

* Police station
* Fire station
* Hospital
* School
* Stadium

These can break up normal frontage and allow different land uses on opposite sides or sections of a street.

---

# 26. Land Value

Land value responds dynamically to:

* Commute time
* Road congestion
* Transit access
* Noise
* Crime
* Schools
* Parks
* Hospitals
* Fire coverage
* Pollution
* Nearby industry
* Highway access
* Walkability
* Parking availability
* Nearby jobs
* Attractions
* Traffic accidents
* Property taxes

Transportation decisions can therefore reshape the economic geography of the city.

---

# 27. Property Development

Private property owners/developers conceptually control normal development.

Development decisions consider:

* Land value
* Zoning
* Density limits
* Taxes
* Accessibility
* Parking
* Neighborhood quality
* Nearby demand

Buildings can redevelop automatically as conditions improve.

Example:

House → townhouse → apartment → dense mixed-use development.

---

# 28. Neighborhoods

Neighborhood identities emerge naturally.

Examples:

* Downtown
* University district
* Industrial corridor
* Wealthy suburb
* Warehouse district
* Entertainment district
* Farming community

Players may rename neighborhoods.

Neighborhoods can have their own ordinances.

---

# 29. Parking

Parking is physically meaningful.

Supported systems:

* Parking lots
* Parking garages
* Street parking
* Paid parking
* Residential parking
* Park-and-ride
* Illegal parking
* Parking shortages

Drivers arriving at a full parking destination search progressively farther away.

The farther someone must park and walk, the more dissatisfied they may become.

Repeated poor parking experiences can make a driver more aggressive or encourage other transportation choices.

---

# 30. Pedestrians

Pedestrians are part of the trip chain.

Example:

Home → sidewalk → bus stop → bus → subway → sidewalk → office.

Individual pedestrians are simulated near the camera.

Distant pedestrian activity can be aggregated.

---

# 31. Sidewalks

Roads can support:

* No sidewalk
* One-sided sidewalk
* Two-sided sidewalk
* Wide sidewalk
* Pedestrian-oriented street
* Crosswalks
* Pedestrian signals

Better pedestrian infrastructure:

* Improves walkability
* Improves accessibility
* Improves safety
* Reduces some accidents
* Can increase land value

---

# 32. Cycling

Cycling is supported by the design.

Possible infrastructure:

* Bike lanes
* Protected bike lanes
* Bike paths
* Shared roads
* Bike parking
* Bike-share infrastructure

Cycling decisions can consider:

* Distance
* Weather
* Safety
* Infrastructure
* Citizen preference

Detailed cycling implementation may be simplified during MVP.

---

# 33. Accessibility

Accessibility affects:

* Walking distance tolerance
* Transit usability
* Crosswalk timing
* Sidewalk requirements
* Parking proximity
* Transfer burden

Good accessibility improves city safety and reduces transportation risk.

---

# 34. Public Transportation

Core modes:

* Bus
* Subway
* Passenger rail
* Freight rail

Transit routes can be manually controlled or AI-managed.

Players can configure:

* Stops
* Routes
* Vehicle count
* Frequency
* Operating hours
* Event schedules
* Express/local operation where applicable

---

# 35. Subway

Characteristics:

* Expensive
* High capacity
* High frequency
* Best for dense districts
* Underground or elevated
* Shorter stop spacing than regional rail

---

# 36. Passenger Rail

Characteristics:

* Longer distance
* Larger stations
* Lower frequency
* Suitable for outer districts

---

# 37. Freight Rail

Characteristics:

* Industrial
* Reduces road freight
* Requires freight terminals
* Creates noise
* Can interact with grade crossings

---

# 38. Underground View

Underground mode does not completely hide surface infrastructure.

Instead:

* Surface roads become very low opacity.
* Underground roads become prominent.
* Subway infrastructure becomes prominent.
* Stations and tunnel connections remain obvious.

This preserves geographic orientation.

---

# 39. Vehicle Types

Core vehicle categories include:

* Personal cars
* Motorcycles
* Taxis
* Rideshare
* Buses
* School buses
* Semi-trucks
* Box trucks
* Delivery vans
* Garbage trucks
* Farm equipment
* Police
* Fire
* Ambulance
* Tow trucks
* Construction vehicles
* Utility/service vehicles

---

# 40. Vehicle Ownership

Households can own specific vehicles.

Ownership may depend on:

* Income
* Household size
* Density
* Parking
* Transit availability
* Policy
* Personal preference

Vehicles may be:

* Gasoline
* Electric
* Other simplified categories if required

---

# 41. Fuel and Charging

Vehicles conceptually track fuel or charge state.

This creates real transportation demand for:

* Gas stations
* EV chargers
* Workplace charging
* Residential charging
* Highway charging locations

EV adoption is primarily influenced through ordinances and policy.

Increasing EV adoption reduces demand for gas stations.

The technology level itself does not evolve historically during a game.

---

# 42. Rideshare and Taxi

Taxis and rideshare vehicles:

* Stage near demand
* Accept passenger requests
* Move between destinations
* Create empty repositioning trips

Large events can create rideshare congestion.

Designated pickup/drop-off areas can reduce the problem.

---

# 43. Freight

Traffic-Jam does **not** simulate a detailed commodity supply-and-demand economy.

Instead:

* Industry produces economic activity and truck traffic.
* Commercial consumes delivery traffic and serves citizens.
* Offices create service-sector employment and economic value.

Business type and size determine generalized delivery activity.

Transportation quality directly affects business performance.

---

# 44. Economy

Businesses can track:

* Revenue
* Payroll
* Employment
* Customer access
* Employee access
* Delivery reliability
* Parking
* Transit access
* Freight access

Poor transportation can cause businesses to:

* Lose revenue
* Reduce employment
* Relocate
* Close

Good transportation improves economic performance.

---

# 45. Farms

Farms:

* Require significant land.
* Produce seasonal traffic.
* Generate heavy vehicle traffic.
* Can attract seasonal workers.
* Can be converted into higher-density development as surrounding land values increase.

---

# 46. External Connections

The simulation ends at city limits.

External connections act as sources and sinks.

Examples:

* Highway
* Regional road
* Rail connection

They can generate:

* Incoming residents
* Visitors
* Commuters
* Freight
* Tourists

The world beyond the boundary is not simulated.

---

# 47. Emergency Services

Players place:

* Police
* Fire
* Hospitals/emergency medical facilities

Emergency vehicles physically travel through the road network.

Traffic yields and slows for them.

Response times therefore depend on actual network design.

---

# 48. Police and Enforcement

Police may be assigned as:

* Traffic enforcement
* Parking enforcement
* Temporary intersection control

More enforcement can:

* Reduce speeding
* Reduce dangerous driving
* Increase ticket revenue
* Reduce citizen happiness

Drivers can remember enforcement locations.

If someone saw police enforcement recently, they may slow down when passing the same area again.

---

# 49. Accidents

Accidents arise from actual conditions such as:

* Speed
* Driver behavior
* Lane changes
* Intersection geometry
* Signal violations
* Weather
* Congestion
* Construction
* Poor road conditions
* Emergency vehicles

Severity can range from:

* Minor crash
* Disabled vehicle
* Injury crash
* Major multi-vehicle crash
* Fatal crash

Crashes physically obstruct roads until cleared.

---

# 50. Disasters and Events

Possible incidents include:

* Vehicle breakdown
* Police chase
* Major crash
* Truck spill
* Train failure
* Subway breakdown
* Traffic signal failure
* Power outage
* Flooding
* Snow
* Ice
* Heavy rain
* Stadium event
* Concert
* Parade
* Protest
* Motorcade
* Sinkhole
* Bridge inspection
* Bridge closure
* Emergency repair
* Severe storm

Some are random.

Others are known ahead of time.

---

# 51. Traffic Control Mode

During major incidents, the player can:

* Close streets
* Close lanes
* Create detours
* Change signals
* Add temporary controls
* Restrict trucks
* Redirect transit
* Request additional emergency vehicles
* Assign police to intersections

AI handles normal dispatch.

The player can intervene when necessary.

---

# 52. Scheduled Traffic Management

Players can create time-dependent configurations.

Examples:

* Planned detours
* Event signal timing
* Stadium traffic plan
* Rush-hour signal timing
* Temporary bus schedules
* Construction routing
* Street restrictions

This allows proactive traffic engineering.

---

# 53. Construction

Infrastructure does not appear instantly after approval.

Construction takes game time.

General target:

**Approximately one day of work per lane for basic construction or modification**, subject to tuning.

Additional complexity adds time.

Examples:

* Parking
* Boulevards
* Medians
* Signals
* Bridges
* Tunnels

---

# 54. Construction Crews

Construction capacity is limited.

Possible crew categories:

* Road
* Signal/electrical
* Bridge
* Rail
* Emergency repair

Projects queue when resources are unavailable.

---

# 55. Contractors

Players can choose among:

* City crew
* Local contractor
* Premium contractor
* Emergency contractor

Tradeoffs include:

* Cost
* Speed
* Reliability
* Availability

---

# 56. Construction Scheduling

Beginner mode can automatically create a reasonable construction plan.

Advanced players can control:

* Start date
* Day/night work
* Weekday/weekend work
* Lane closures
* Construction phases
* Detours
* Temporary signals
* Temporary limits
* Police presence
* Target completion

---

# 57. Construction Risk

Projects may encounter:

* Weather
* Contractor delays
* Ground problems
* Material shortages
* Work-zone crashes
* Complaints
* Budget overruns
* Failed inspections
* Emergency crew reassignment

---

# 58. Planning Mode

Planning Mode is a major feature.

Before committing a project, the player can temporarily apply proposed infrastructure changes.

The simulation behaves as though the project is complete.

The planning state is **not saved as the real city state**.

Players can:

* Run traffic.
* Compare congestion.
* Inspect commute changes.
* Test intersection timing.
* Modify the proposal.
* Cancel freely.
* Approve when satisfied.

---

# 59. Plan Comparison

Major projects can display:

**Current vs Proposed**

Metrics can include:

* Average commute
* Traffic delay
* Intersection efficiency
* Crash risk
* Transit delay
* Parking behavior
* Construction cost
* Maintenance change
* Land-value impact
* Construction duration

---

# 60. Undo and Demolition

During planning:

**Undo is extremely forgiving.**

Once infrastructure is physically constructed:

There is no magical instant undo.

Removing or changing completed infrastructure requires:

* Demolition
* Construction time
* Cost
* Traffic disruption

---

# 61. Infrastructure Condition

Basic states:

* Excellent
* Good
* Worn
* Poor
* Critical

Condition depends on:

* Traffic volume
* Heavy vehicles
* Weather
* Age
* Maintenance
* Construction quality

---

# 62. Maintenance Budget

Possible budget categories:

* Potholes
* Resurfacing
* Bridges
* Signals
* Snow removal
* Transit
* Markings/signs
* Emergency repair reserve

Players can use high-level budget allocation and individual project decisions.

Higher sustained infrastructure funding:

* Slows deterioration.
* Reduces major failures.
* Shortens repairs.
* Improves road health.

---

# 63. Repair Quality

Players can generally choose:

* Cheap
* Balanced
* Premium

Cheap repairs cost less now but may deteriorate sooner.

Premium work lasts longer.

---

# 64. Weather

Weather includes:

* Rain
* Heavy rain
* Snow
* Ice
* Fog
* Heat
* Flooding
* Severe storms

Weather affects:

* Vehicle speed
* Crash risk
* Transit
* Construction
* Road condition
* Emergency response
* Events

---

# 65. Snow Operations

Players can prioritize:

* Arterials
* Emergency routes
* School routes
* Transit routes
* Residential roads

Possible controls:

* Plow priority
* Salt/brine
* Parking bans
* Overnight restrictions

Parked vehicles can interfere with snow clearing.

---

# 66. Engineering Budget

The player receives a **weekly engineering budget**.

The player's budget is a portion of the overall city budget.

The player controls how engineering resources are spent.

Examples:

* Maintenance
* Potholes
* New streets
* Road expansion
* Transit
* Signals
* Bridges
* Emergency projects

Better city performance can result in a larger allocation.

---

# 67. Mayor

The mayor behaves somewhat like a strategic Civilization-style leader.

Potential mayor priorities:

* Growth
* Environment
* Business
* Transit
* Low taxes
* Safety
* Suburban development

Mayor personality can affect:

* Infrastructure allocation
* Ordinance support
* City priorities
* Objectives

---

# 68. Elections

A mayoral election occurs approximately once per year.

The player's political influence is based on engineering performance.

The player's vote carries more influence when the player is performing well.

This influence does not allow a poorly performing player to simply force unpopular decisions.

---

# 69. Ordinances

Examples:

* Parking fees
* Congestion pricing
* Rush-hour parking restrictions
* Truck restrictions
* Truck delivery hours
* HOV lanes
* Bus-only lanes
* Speed limits
* School-zone policy
* Turn restrictions
* EV incentives
* Free transit
* Transit fares
* Residential parking permits
* Parking minimums
* Remove parking minimums
* Cycling incentives
* Work-from-home incentives

Neighborhood-specific ordinances are supported.

---

# 70. Ordinance Approval

Approval depends on:

* Citizens
* Council
* Happiness
* City conditions
* Political environment

The player sees estimated approval probability.

A successful engineering record improves political influence.

A poorly regarded engineer trends toward much less reliable approval rather than gaining easy access to reverse policies.

Ordinances have political consequences.

---

# 71. City Score

The city has an overall engineering/performance score.

Components include:

* Traffic efficiency
* Safety
* Citizen happiness
* Economic health
* Transit performance
* Infrastructure health
* Environmental impact
* Growth

Detailed category scores remain visible underneath the overall score.

---

# 72. Traffic Efficiency

Congestion should be measured against expected free-flow conditions.

Metrics may consider:

* Speed relative to expected speed
* Queue length
* Intersection delay
* Travel-time inflation
* Capacity utilization

A busy road is not automatically congested if it efficiently moves its traffic.

---

# 73. Intersection Score

Each intersection can expose:

* Efficiency score
* Vehicles/hour
* Average delay
* Maximum queue
* Capacity utilization
* Crash rate
* Turn movements
* Truck percentage
* Transit delay
* Emergency delay
* Peak periods

---

# 74. Wasted Movement

The game tracks inefficient transportation.

Examples:

* Parking search
* Empty rideshare travel
* Excessive truck detours
* Buses stuck in traffic
* Poor transfers
* Bad intersection routing
* Emergency delay

Reducing wasted movement is a major optimization objective.

---

# 75. Heatmaps

Supported heatmaps include:

* Congestion
* Average speed
* Crash risk
* Crashes
* Driver frustration
* Noise
* Pollution
* Land value
* Parking demand
* Transit demand
* Transit coverage
* Emergency response
* Freight
* Pedestrian activity
* Road condition
* Construction impact
* Density
* Jobs
* Population

---

# 76. Analytics

Analytics support:

* Current conditions
* Daily comparison
* Weekly comparison
* Monthly comparison
* Historical trends

Players should be able to identify the worst-performing:

* Roads
* Intersections
* Districts
* Transit routes

This supports Factorio-style bottleneck hunting.

---

# 77. Citizen Happiness

Happiness is influenced by categories including:

* Commute
* Housing cost
* Job
* Safety
* Taxes
* Parking
* Transit
* Schools
* Parks
* Noise
* Pollution
* Police
* Services

Both overall happiness and category-level causes are visible.

---

# 78. Social Media Feed

The social feed provides human-readable hints generated from actual simulation events.

Examples:

“Whoever retimed the lights on Chester deserves a raise.”

“Another twenty minutes looking for parking downtown.”

“The bus was late again.”

“Traffic by the stadium is ridiculous tonight.”

Posts can be humorous or exaggerated in tone.

They must not describe events that did not actually occur.

Clicking a relevant post should take the player toward the location or system involved.

---

# 79. Calendar

The game includes:

* Weekly calendar
* Monthly calendar
* Annual calendar

The calendar displays upcoming:

* Events
* Construction
* Inspections
* School schedules
* Weather
* Closures
* Transit changes

It also manages:

* Signal timing schedules
* Transit schedules
* Detours
* Event plans
* Automation

Selecting a street, intersection, or corridor can reveal its scheduled timing plans.

---

# 80. Reports

Monthly reports include:

* Population change
* City score
* Commute
* Delay
* Crashes
* Fatalities
* Tickets
* Happiness
* Transit
* Roads
* Construction
* Emergency response
* Land values
* Business activity
* Revenue
* Engineering budget
* Ordinance results
* Neighborhood issues

Reports compare:

* Previous month
* Prior-year period when available

---

# 81. Visual Style

Traffic-Jam uses a clean **vector-map visual language** inspired by the usability of modern navigation maps.

It is not a satellite-view game.

Visual characteristics:

* Light neutral land
* Green park/open space
* Blue water
* Clear road hierarchy
* Minimal labels
* Strong overlays
* Clean topology

The game should develop its own visual identity rather than copying Google Maps directly.

---

# 82. Buildings

There are effectively no traditional rendered buildings.

Properties are represented by:

* Zone color
* Parcel/frontage
* Density

Higher-density development uses greater color saturation.

This greatly reduces rendering complexity.

---

# 83. Vehicles

At close zoom:

* Cars = dots
* Commercial trucks = rectangles
* Taxis/rideshare = yellow
* Buses = yellow rectangles
* Emergency/service vehicles receive distinctive visual treatment

At distant zoom:

Individual vehicles progressively disappear.

Traffic becomes aggregate flow visualization.

---

# 84. Strategic Zoom

At maximum zoom-out, emphasize:

* Major streets
* Highways
* Rail
* Subway
* Neighborhood boundaries
* Congestion
* Land values
* Population
* Construction
* Major incidents
* Special destinations

---

# 85. Audio

Audio remains restrained.

Possible sounds:

* City ambience
* Highway noise
* Rail
* Subway
* Sirens
* Construction
* Rain
* Storms
* UI alerts
* Ambient soundtrack

Physical city sounds fade as the player zooms out.

---

# 86. Difficulty

## Easy

Focus:

* Relaxed city building
* More forgiving budget
* Patient drivers
* Slower deterioration
* Fewer severe incidents

## Normal

Balanced simulation.

## Hard

Focus:

* Active traffic management
* More disasters
* Tighter budgets
* More difficult congestion
* Greater infrastructure pressure
* Less forgiving driver behavior

## Expert

Maximum simulation pressure.

Difficulty should change legitimate system parameters rather than giving AI actors arbitrary advantages.

---

# 87. Failure

The player can lose.

Potential failure states include:

* Severe financial collapse
* Population collapse
* Critical happiness
* Removal by government
* Unsafe infrastructure crisis
* City fiscal emergency

---

# 88. Tutorial

Optional guided first city.

Suggested progression:

1. Build road.
2. Zone land.
3. Create intersection.
4. Configure light.
5. Fix traffic problem.
6. Add transit.
7. Handle accident.
8. Manage construction.
9. Propose ordinance.
10. Read analytics.

---

# 89. AI Automation

Systems can support automation where appropriate.

Examples:

* Signals
* Transit
* Maintenance
* Construction
* Snow
* Traffic response

Automation should not receive an artificial penalty.

AI should be **competent and conservative**.

Skilled human players may outperform it through more creative optimization.

---

# 90. Technology

The game does not require a technology tree.

Technology does not fundamentally advance through historical eras.

EV adoption can change through policy.

Autonomous vehicles may have distinct behavior:

* More patient
* Safer
* Better merging
* Better routing
* More conservative

Connected smart-intersection infrastructure is a later project.

---

# 91. Map Generation

Maps are procedurally generated from seeds.

Possible elements:

* River
* Lake
* Regional road connections
* Existing small town
* Farmland
* Industrial areas
* Parks

Terrain elevation remains intentionally simple.

Generation focuses on transportation constraints.

---

# 92. Starting Presets

Possible starting configurations:

* Rural town
* Suburban sprawl
* Dense core
* Industrial city
* Transit-heavy city
* Highway-dependent city

The default start should contain a small functioning settlement so the player immediately has something to analyze and improve.

---

# 93. City Expansion

City limits expand automatically as development approaches the boundary.

Players do not purchase map tiles individually.

---

# 94. Saves

Support:

* Manual save
* Autosave
* Multiple saves
* Named checkpoints
* Planning snapshots

Configuration and metadata can use human-readable formats such as RON or JSON.

Large simulation state should use efficient compressed/binary storage.

---

# 95. Technical Architecture

Primary stack:

**Rust + Bevy**

Development priorities:

1. Simulation correctness
2. Simulation scale
3. CPU efficiency
4. Memory efficiency
5. Usability
6. Graphics

Bevy's ECS model can be used where beneficial, but not every statistical citizen should necessarily become a permanent ECS entity.

Large simulation systems should favor:

* Data-oriented storage
* Batching
* Spatial partitioning
* Parallel jobs
* Aggregation
* Fixed simulation ticks
* Decoupled rendering
* Simulation LOD

---

# 96. Platform Strategy

Initial platform:

**Linux**

Steam Deck must be considered during architecture and performance work.

Later desktop support may include:

* Windows
* macOS

Steam Deck performance should influence:

* Memory budgets
* Active-agent limits
* Rendering
* UI size
* Simulation scaling

---

# 97. Performance Targets

Target:

**60 FPS** during normal interaction where practical.

A temporary reduction toward 30 FPS may be acceptable under extreme close-up load.

Simulation should continue prioritizing correctness and scale.

Primary population ranges:

* Normal: 10k–100k
* Large: hundreds of thousands+
* Stress target: 10 million

Ten million residents will require extreme statistical aggregation.

---

# 98. Mod Architecture

Mod support is not required for MVP but should influence architecture.

Future mod targets:

* Ordinances
* Roads
* Vehicles
* Events
* Scenarios
* Maps
* Overlays
* Simulation configuration

Prefer data-driven mod APIs over arbitrary native Rust code.

---

# 99. MVP

The first playable version should establish the entire gameplay loop at small scale.

Required:

* Procedural starting town
* Road drawing
* Road editing
* Intersections
* Traffic signals
* Stop signs
* Basic zoning
* Residential
* Commercial
* Office
* Industry
* Farm
* Basic citizens
* Home/work commuting
* Cars
* Trucks
* Buses
* Emergency vehicles
* Parking
* Congestion
* Accidents
* Breakdowns
* Emergency response
* Weekly budget
* Basic maintenance
* Planning Mode
* Traffic heatmap
* Road analytics
* Intersection analytics
* Basic social feed
* Save/load

---

# 100. MVP Simplifications

These systems may initially be simplified:

* Subway
* Passenger rail
* Freight rail
* Contractor complexity
* Detailed weather
* Advanced ordinances
* Elections
* Mayor personalities
* Detailed pedestrians
* Cycling
* Police enforcement depth
* Emergent neighborhood identity
* Advanced mixed-use redevelopment
* Complex seasonal agriculture

Their architectural requirements should still be considered.

---

# 101. Development Order

## Milestone 1 — Tiny Complete City

Highest priority.

The player can:

1. Build roads.
2. Zone land.
3. Watch housing/jobs appear.
4. Watch citizens commute.
5. Create congestion.
6. Inspect the problem.
7. Modify the network.
8. Observe the result.

Graphics can be extremely primitive.

This milestone proves the game.

## Milestone 2 — Road Editor

Improve:

* Geometry
* Editing
* Lanes
* Intersections
* Road classes
* Planning tools

## Milestone 3 — Traffic Simulation

Improve:

* Lane behavior
* Routing
* Driver memory
* Parking
* Intersections
* Traffic adaptation
* Incidents

## Milestone 4 — Analytics

Improve:

* Heatmaps
* Bottleneck identification
* Historical reporting
* Plan comparison
* Efficiency scoring

---

# 102. Recommended First Development Prototype

The absolute first vertical slice should intentionally look crude.

Suggested target:

**Population: 500–2,000**

Required loop:

1. Blank/simple map.
2. Draw two roads.
3. Create intersection.
4. Zone homes.
5. Zone jobs.
6. Residents appear.
7. Residents drive to work.
8. Traffic light controls intersection.
9. Congestion develops if capacity is insufficient.
10. Player modifies intersection.
11. Traffic improves.
12. Analytics prove the improvement.

Once that works reliably, scale it.

Do **not** begin development with:

* Beautiful UI
* Subway
* Elections
* Weather
* Detailed construction
* Multiplayer
* Advanced police
* Ten-million-person simulation

Prove the transportation loop first.

---

# 103. Core Success Criterion

Traffic-Jam succeeds if the player can spend an hour staring at one part of the city thinking:

**“I know I can make this intersection 5% better.”**

Then redesign it, test it, deploy it, and watch the entire surrounding transportation system react.

A successful city should eventually produce the satisfaction of watching a highly optimized factory:

* Cars flow.
* Signals synchronize.
* Buses arrive on time.
* Trucks stay on freight corridors.
* Emergency vehicles get through.
* Parking is available.
* Citizens arrive on time.
* Accidents are rare.
* Construction is planned.
* Traffic jams disappear.

And then the city grows enough to create the next problem.

---

# 104. Long-Term Vision

Traffic-Jam should ultimately support cities that feel substantially larger than traditional city builders because graphical building simulation is intentionally deprioritized.

The long-term identity is:

**A massive-scale civil engineering and transportation optimization simulator presented through a clean interactive map.**

The city is not primarily something the player decorates.

The city is something the player **debugs, tunes, expands, and optimizes.**

---

# 105. Scope Classification

## MVP

Build the tiny complete loop and prove:

**Road → Zone → Citizen → Trip → Congestion → Analysis → Redesign → Improvement**

## Phase 2

Expand:

* Construction
* Maintenance
* Parking
* Emergency response
* Transit
* Weather
* Police
* Ordinances
* Neighborhoods
* Larger populations

## Phase 3

Expand:

* Subway
* Rail
* Freight
* Deep events
* Elections
* Advanced economy
* Large-scale city management
* Advanced automation

## Stretch Goals

* Meta-progression
* Advanced smart-city systems
* Extensive mod tools
* Scenario editor
* Scenario campaign

## Future Project

Multiplayer, including hosted and player-hosted servers.

Multiplayer considerations should influence boundaries between simulation, rendering, input, and saved city state from the beginning, but multiplayer itself must not delay the single-player game.

---

# 106. One-Sentence Pitch

**Traffic-Jam is a Rust-based city transportation simulator where you engineer an endlessly growing city, optimize every movement from individual traffic lights to metropolitan transit networks, and try to achieve Factorio-level efficiency without making your citizens miserable.**

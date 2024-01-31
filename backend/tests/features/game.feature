Feature: Games

  Scenario: Creating a game requires root team
    Given a system S1 created by user U1 owned by team T1
    When user U1 creates a game G1 on system S1
    Then an error occured

  Scenario: Can add a game to a system
    Given a system S1 created by user U1 owned by team T1
    When admin A1 creates a game G1 on system S1
    Then no error occured
    And game G1 exists on system S1

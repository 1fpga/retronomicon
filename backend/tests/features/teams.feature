Feature: Team

  Scenario: Can create a team
    Given team C is owned by user A

  Scenario: Can invite a user to a team
    Given team C is owned by user A
    When user A invites user B to team C as member
    And  user B accepts the invitation to team C
    Then team C will have user B as member
    Then team C will have user A as owner

  Scenario: Cannot invite a user to a team if not owner
    Given team T1 is owned by user A
    When user B invites user C to team T1 as member
    Then an error occured

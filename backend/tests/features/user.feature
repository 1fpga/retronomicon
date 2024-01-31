Feature: User

  Scenario: User can see its own info
    Given user A is not authenticated
    When user A gets their details
    Then no error occured

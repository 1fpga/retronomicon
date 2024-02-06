Feature: Game Images

  Scenario: Admin can upload game images
    Given game G1
    When admin A1 uploads image I1 to game G1
    Then no error occured
     And anonymous user can see image I1 of game G1
     And admin A1 can see image I1 of game G1

  Scenario: Only admin can upload game images
    Given game G1
    When user U1 uploads image I1 to game G1
    Then an error occured
